use anyhow::{Context, Ok};
use aptos_crypto::{SigningKey, ValidCryptoMaterialStringExt, ed25519::*, traits::signing_message};
use aptos_sdk::coin_client::TransferOptions;
use aptos_sdk::rest_client::Client;
use aptos_sdk::rest_client::aptos_api_types::ViewRequest;
use aptos_sdk::transaction_builder::TransactionBuilder;
use aptos_sdk::types::account_address::AccountAddress;
use aptos_sdk::types::chain_id::ChainId;
use aptos_sdk::types::transaction::SignedTransaction;
use aptos_sdk::types::transaction::authenticator::{AccountAuthenticator, AuthenticationKey};
use aptos_sdk::types::transaction::{RawTransactionWithData, TransactionPayload};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{str::FromStr, sync::Arc};
use turnkey_client::generated::immutable::common::v1::{
    AddressFormat, Curve, HashFunction, PathFormat, PayloadEncoding,
};
use turnkey_client::generated::{
    CreateWalletIntent, ExportWalletAccountIntent, SignRawPayloadIntentV2, WalletAccountParams,
};
use turnkey_client::{TurnkeyClient, TurnkeyP256ApiKey};
use turnkey_enclave_encrypt::{ExportClient, QuorumPublicKey};

use crate::config::Config;

pub struct AptosClient {
    client: Client,
    turnkey: TurnkeyClient<TurnkeyP256ApiKey>,
    config: Arc<Config>,
    chain_id: ChainId,
}

impl AptosClient {
    pub async fn new(config: Arc<Config>) -> anyhow::Result<Self> {
        // aptos
        let client = Client::new(url::Url::from_str(&config.aptos_base_url)?);
        // turnkey
        let turnkey_api_key = TurnkeyP256ApiKey::from_strings(
            config.turnkey_config.api_private_key.clone(),
            Some(config.turnkey_config.api_public_key.to_string()),
        )
        .context("Failed to get turnkey p256 api key")?;

        let turnkey = TurnkeyClient::builder()
            .api_key(turnkey_api_key)
            .build()
            .context("Failed to build turnkey client")?;

        let chain_id = client.get_index().await?.inner().chain_id;

        Ok(Self {
            client,
            turnkey,
            config,
            chain_id: ChainId::new(chain_id),
        })
    }

    pub async fn create_new_wallet_on_turnkey(
        self: &Arc<Self>,
        wallet_name: &str,
    ) -> anyhow::Result<(String, String, String)> {
        let create_wallet_result = self
            .turnkey
            .create_wallet(
                self.config.turnkey_config.organization_id.clone(),
                self.turnkey.current_timestamp(),
                CreateWalletIntent {
                    wallet_name: wallet_name.into(),
                    accounts: vec![WalletAccountParams {
                        curve: Curve::Ed25519,
                        path_format: PathFormat::Bip32,
                        path: "m/44'/60'/0'/0".to_string(),
                        address_format: AddressFormat::Aptos,
                    }],
                    mnemonic_length: None,
                },
            )
            .await?;

        assert_eq!(create_wallet_result.addresses.len(), 1);

        let wallet_address = create_wallet_result
            .addresses
            .first()
            .context("Failed to get first wallet address")?;
        let wallet_id = create_wallet_result.wallet_id;
        let private_key = Self::export_private_key(&Arc::clone(&self), &wallet_address).await?;
        let public_key = Self::get_public_key_str_from_private_key_str(&private_key)?;
        Ok((wallet_id, wallet_address.clone(), public_key))
    }

    pub async fn export_private_key(
        self: &Arc<Self>,
        wallet_address: &str,
    ) -> anyhow::Result<String> {
        let mut export_client = ExportClient::new(&QuorumPublicKey::production_signer());
        let export_wallet_result = self
            .turnkey
            .export_wallet_account(
                self.config.turnkey_config.organization_id.clone(),
                self.turnkey.current_timestamp(),
                ExportWalletAccountIntent {
                    address: wallet_address.into(),
                    target_public_key: export_client.target_public_key()?,
                },
            )
            .await?;

        let export_bundle = export_wallet_result.export_bundle;
        let private_key_bytes = export_client.decrypt_private_key(
            export_bundle,
            self.config.turnkey_config.organization_id.clone(),
        )?;

        let private_key = hex::encode(&private_key_bytes);
        Ok(private_key)
    }

    fn get_public_key_str_from_private_key_str(private_key: &str) -> anyhow::Result<String> {
        let mut seed = [0u8; 32];
        let seed_bytes = hex::decode(private_key).context("Failed to decode private key string")?; // Remove the 0x prefix
        seed[..seed_bytes.len()].copy_from_slice(&seed_bytes);
        let sender_key = Ed25519PrivateKey::try_from(seed_bytes.as_slice())?;

        let sender_public_key = Ed25519PublicKey::from(&sender_key);
        let sender_public_key_str = sender_public_key.to_string();
        Ok(sender_public_key_str)
    }

    pub async fn sign_submit_txn_with_turnkey_and_fee_payer(
        &self,
        sender_address: &str,
        sender_public_key: &str,
        payload: TransactionPayload,
    ) -> anyhow::Result<String> {
        let options = TransferOptions::default();
        let sender = AccountAddress::from_hex_literal(sender_address)?;

        let sequence_number = self.client.get_account_sequence_number(sender).await?;
        let raw_txn = TransactionBuilder::new(
            payload,
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + options.timeout_secs,
            self.chain_id.clone(),
        )
        .sender(sender)
        .sequence_number(*sequence_number.inner())
        .build();

        let mut seed = [0u8; 32];
        let seed_bytes = hex::decode(&self.config.admin_config.sponsor_private_key)?;
        seed[..seed_bytes.len()].copy_from_slice(&seed_bytes);

        let sponsor_key = Ed25519PrivateKey::try_from(seed_bytes.as_slice())?;
        let sponsor_address =
            AuthenticationKey::ed25519(&Ed25519PublicKey::from(&sponsor_key)).account_address();

        let message =
            RawTransactionWithData::new_fee_payer(raw_txn.clone(), vec![], sponsor_address);
        let sponsor_signature = sponsor_key.sign(&message).unwrap();
        let sponsor_authenticator =
            AccountAuthenticator::ed25519(Ed25519PublicKey::from(&sponsor_key), sponsor_signature);

        let signing_message = signing_message(&message)?;
        let txn_sign_result = self
            .turnkey
            .sign_raw_payload(
                self.config.turnkey_config.organization_id.clone(),
                self.turnkey.current_timestamp(),
                SignRawPayloadIntentV2 {
                    sign_with: sender.to_string(),
                    payload: hex::encode(signing_message),
                    encoding: PayloadEncoding::Hexadecimal,
                    hash_function: HashFunction::NotApplicable,
                },
            )
            .await?;

        let r_hex = format!("{:0>64}", txn_sign_result.r);
        let s_hex = format!("{:0>64}", txn_sign_result.s);

        let txn_signature_hex = format!("{}{}", r_hex, s_hex);

        assert_eq!(txn_signature_hex.len(), 128);

        let sig_bytes = hex::decode(txn_signature_hex).unwrap();

        assert_eq!(sig_bytes.len(), 64);

        let sender_signature = Ed25519Signature::try_from(sig_bytes.as_slice())?;

        let sender_authenticator: AccountAuthenticator = AccountAuthenticator::ed25519(
            Ed25519PublicKey::from_encoded_string(sender_public_key).unwrap(),
            sender_signature,
        );

        let signed_txn = SignedTransaction::new_fee_payer(
            raw_txn,
            sender_authenticator.clone(),
            vec![],
            vec![],
            sponsor_address,
            sponsor_authenticator,
        );

        let pending_transaction = self.client.submit(&signed_txn).await?;

        self.client
            .wait_for_transaction(pending_transaction.inner())
            .await?;

        Ok(pending_transaction.inner().hash.to_string())
    }

    pub async fn view(&self, request: &ViewRequest) -> anyhow::Result<Vec<serde_json::Value>> {
        let response = self.client.view(request, None).await?;
        Ok(response.into_inner())
    }
}

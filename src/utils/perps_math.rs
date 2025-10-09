use bigdecimal::BigDecimal;

pub fn notional_price(collateral: &BigDecimal, leverage: u8) -> BigDecimal {
    let leverage = BigDecimal::from(leverage);
    collateral * &leverage
}

pub fn position_size(notional_price: &BigDecimal, entry_price: &BigDecimal) -> BigDecimal {
    notional_price / entry_price
}

fn initial_margin_ratio(leverage: u8) -> BigDecimal {
    let leverage = BigDecimal::from(leverage);
    BigDecimal::from(1) / leverage
}

pub fn liquidation_price(is_long: bool, entry_price: &BigDecimal, leverage: u8, maintenance_margin_ratio: &BigDecimal) -> BigDecimal {
    let initial_margin_ratio = initial_margin_ratio(leverage);
    let one = BigDecimal::from(1);
    let multiplier = if is_long {
        one - initial_margin_ratio + maintenance_margin_ratio
    } else {
        one + initial_margin_ratio - maintenance_margin_ratio
    };

    entry_price * multiplier
}

pub fn position_value(position_size: &BigDecimal, price: &BigDecimal) -> BigDecimal {
    position_size * price
}
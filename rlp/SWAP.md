The main formula for swap:

Current Oracle based only formula:
Oracle formula: oracle_out = a × y / x

We're searching to define an "impact factor" that defines a relationship between
the trade size and liquidity in the pool, it can never exceed 1 so we can apply it to the oracle price as such:

amount_out = oracle_amount_out × (1 - impact_factor)

The impact factor should be high for a big trade size and low pool liquidity and low for a small trade with high pool liquidity:

Linear:
impact_factor = trade_size / (liquidity_pool)

Problem: What if user wants to swap 200% of pool? impact = 2 so:
amount_out = oracle_out * (1 - 2) = oracle_out * (-1) = negative!

Asymptotic:
impact_factor = trade_size / (liquidity_pool + trade_size)

advantage: it can never exceed 1 so our formula is safe.

Relation to AMM constant product:

x = amount of token x in the pool
y = amount of token y in the pool
a = amount of token x that wants to be swapped

Following the AMM constant product formula, K = x * y we get:

amount_out = y × a / (x + a)

we're looking to express the AMM out formula as:

amount_out = something × (1 - impact_factor)

If we divide amm_amount_out / oracle_amount_out we get:

amount_out       y × a / (x + a)
──────────── = ───────────────── = x / (x + a)
oracle_out         a × y / x

This means:

amount_out = oracle_out × x/(x + a)
           = oracle_out × (1 - impact_factor)

Therefore:

1 - impact_factor = x / (x + a)

impact_factor = 1 - x/(x + a)
              = (x + a - x) / (x + a)
              = a / (x + a)


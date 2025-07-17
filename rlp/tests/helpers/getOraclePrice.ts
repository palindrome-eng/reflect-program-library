import axios from "axios";
import BN from "bn.js";

type Response = {
    id: "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
    price: {
        price: `${number}`,
        conf: `${number}`,
        expo: number,
        publish_time: number
    },
    ema_price: {
        price: `${number}`,
        conf: `${number}`,
        expo: number,
        publish_time: number
    }
}[]

type OraclePrice = {
    price: BN,
    precision: BN
};

const priceMap: Map<string, OraclePrice> = new Map([]);

export default async function getOraclePrice(feed: string = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"): Promise<OraclePrice> {
    if (priceMap.has(feed)) {
        return priceMap.get(feed);
    }

    const {
        data
    } = await axios.get<Response>(`https://hermes.pyth.network/api/latest_price_feeds?ids[]=${feed}`);

    const {
        price: {
            price,
            expo
        }
    } = data[0];

    const oraclePrice: OraclePrice = {
        price: new BN(price),
        precision: new BN(expo).abs()
    };

    priceMap.set(feed, oraclePrice);

    return oraclePrice;
}
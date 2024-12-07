"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.accountProviders = void 0;
__exportStar(require("./Admin"), exports);
__exportStar(require("./Asset"), exports);
__exportStar(require("./Cooldown"), exports);
__exportStar(require("./Deposit"), exports);
__exportStar(require("./Intent"), exports);
__exportStar(require("./Lockup"), exports);
__exportStar(require("./RewardBoost"), exports);
__exportStar(require("./Settings"), exports);
__exportStar(require("./Slash"), exports);
const Admin_1 = require("./Admin");
const Asset_1 = require("./Asset");
const Cooldown_1 = require("./Cooldown");
const Deposit_1 = require("./Deposit");
const Intent_1 = require("./Intent");
const Lockup_1 = require("./Lockup");
const RewardBoost_1 = require("./RewardBoost");
const Settings_1 = require("./Settings");
const Slash_1 = require("./Slash");
exports.accountProviders = {
    Admin: Admin_1.Admin,
    Asset: Asset_1.Asset,
    Cooldown: Cooldown_1.Cooldown,
    Deposit: Deposit_1.Deposit,
    Intent: Intent_1.Intent,
    Lockup: Lockup_1.Lockup,
    RewardBoost: RewardBoost_1.RewardBoost,
    Settings: Settings_1.Settings,
    Slash: Slash_1.Slash,
};

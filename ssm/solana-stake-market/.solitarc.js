const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/solana-stake-market');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'solana_stake_market',
    programId: 'sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
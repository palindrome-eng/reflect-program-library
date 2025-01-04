const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/insurance-fund');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk/src/generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'insurance_fund',
    programId: '2MN1Dbnu7zM9Yj4ougn6ZCNNKevrSvi9AR56iawzkye8',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
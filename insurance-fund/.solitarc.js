const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/insurance-fund');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk/src/generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'insurance_fund',
    programId: 'EiMoMLXBCKpxTdBwK2mBBaGFWH1v2JdT5nAhiyJdF3pV',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
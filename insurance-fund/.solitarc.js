const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/insurance-fund');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk/generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'insurance_fund',
    programId: 'CPW6gyeGhh7Kt3LYwjF7yXTYgbcNfT7dYBSRDz7TH5YB',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
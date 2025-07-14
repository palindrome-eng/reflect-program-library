const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/rlp');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk/src/generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'rlp',
    programId: 'rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
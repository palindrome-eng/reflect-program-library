const path = require('path');
const programDir = path.join(__dirname, '.', 'programs/reflect-tokenised-bonds');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'sdk/src');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'reflect_tokenised_bonds',
    programId: '6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
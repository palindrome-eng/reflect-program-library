import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor as renderJsVisitor } from "@codama/renderers-js";
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";
import { visit } from "@codama/visitors-core";
import { readFileSync } from "fs";
import path from "path";

const idlPath = path.join(__dirname, "target", "idl", "rlp.json");
const idl = JSON.parse(readFileSync(idlPath, "utf-8"));

const rootNode = rootNodeFromAnchor(idl);

// Generate TypeScript SDK
const jsOutputDir = path.join(__dirname, "sdk", "src", "generated");
visit(rootNode, renderJsVisitor(jsOutputDir));

// Generate Rust SDK
const rustCrateDir = path.join(__dirname, "clients", "rust", "src", "generated");
visit(rootNode, renderRustVisitor(rustCrateDir, {
  formatCode: true
}));

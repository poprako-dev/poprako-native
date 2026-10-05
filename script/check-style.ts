import { readdir, readFile } from "node:fs/promises";
import { basename, join } from "node:path";
import ts from "typescript";
import { run } from "./process.ts";

async function files(directory: string): Promise<string[]> {
  const result: string[] = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (["generated", "target", "gen"].includes(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await files(path)));
    if (entry.isFile() && /\.tsx?$/.test(path) && !path.endsWith(".gen.ts"))
      result.push(path);
  }
  return result;
}

const errors: string[] = [];
const paths = [...(await files("src")), ...(await files("script"))];
for (const path of paths) {
  const text = await readFile(path, "utf8");
  const name = basename(path);
  const lines = text.split("\n").length - Number(text.endsWith("\n"));
  if (lines > 400)
    errors.push(`${path}: ${String(lines)} physical lines exceeds 400`);
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true);
  const isRoute =
    /(?:^|[\\/])route[\\/]/.test(path) && !/[\\/]business[\\/]/.test(path);
  if (
    !isRoute &&
    name.endsWith(".tsx") &&
    !/^[A-Z][a-zA-Z0-9]*(?:\.test)?\.tsx$/.test(name)
  )
    errors.push(`${path}: TSX requires PascalCase`);
  if (
    name.endsWith(".ts") &&
    !(
      name.endsWith(".test.ts") &&
      paths.includes(path.replace(".test.ts", ".tsx"))
    ) &&
    !/^[a-z][a-z0-9-]*(?:\.test|\.d)?\.ts$/.test(name)
  )
    errors.push(`${path}: TS requires kebab-case`);
  function visit(node: ts.Node): void {
    if (ts.isInterfaceDeclaration(node) && !ts.isModuleBlock(node.parent)) {
      const callable =
        node.members.length > 0 &&
        node.members.every(
          (member) =>
            ts.isMethodSignature(member) ||
            ts.isCallSignatureDeclaration(member) ||
            (ts.isPropertySignature(member) &&
              member.type !== undefined &&
              ts.isFunctionTypeNode(member.type)),
        );
      if (node.name.text.endsWith("Props") || !callable)
        errors.push(
          `${path}: data and Props must use type (${node.name.text})`,
        );
    }
    if (
      ts.isTypeAliasDeclaration(node) &&
      !node.name.text.endsWith("Props") &&
      ts.isTypeLiteralNode(node.type) &&
      node.type.members.length > 0 &&
      node.type.members.every(
        (member) =>
          ts.isMethodSignature(member) ||
          ts.isCallSignatureDeclaration(member) ||
          (ts.isPropertySignature(member) &&
            member.type !== undefined &&
            ts.isFunctionTypeNode(member.type)),
      )
    )
      errors.push(
        `${path}: pure callable contract must use interface (${node.name.text})`,
      );
    if (!isRoute && node.kind === ts.SyntaxKind.ExportAssignment)
      errors.push(`${path}: source modules use named exports`);
    ts.forEachChild(node, visit);
  }
  visit(source);
}
if (errors.length > 0) throw new Error(errors.join("\n"));
await run("deno", ["task", "check:rust-style"]);
console.log("Project style checks passed.");

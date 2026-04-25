// Entrypoint for converting backend-generated JSON Schemas into frontend Zod schemas.
// Custom parser overrides keep app-specific behavior, like CalendarDate form values and
// backend-owned validation messages, out of the Vue form layer.

import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import $RefParser from "@apidevtools/json-schema-ref-parser";
import { jsonSchemaToZod } from "json-schema-to-zod";

import { dateParserOverride } from "./generate-zod.date.mjs";
import { createValidationMessagesParserOverride } from "./generate-zod.validation.mjs";

const schemaDir = join(process.cwd(), "app", "generated", "schemas");
const outputDir = join(process.cwd(), "app", "generated", "zod");

const schemaSuffix = ".schema.json";

function toCamelCase(name) {
  if (!name.length) return name;
  return `${name[0].toLowerCase()}${name.slice(1)}`;
}

const validationMessagesParserOverride = createValidationMessagesParserOverride(() => parserOverride);

function parserOverride(schemaNode, refs) {
  return dateParserOverride(schemaNode, refs) ?? validationMessagesParserOverride(schemaNode, refs);
}

async function run() {
  await mkdir(outputDir, { recursive: true });

  const schemaFiles = (await readdir(schemaDir))
    .filter((file) => file.endsWith(schemaSuffix))
    .sort();

  const indexLines = [
    "// Generated file, update with `bun run contracts:gen`.",
    ""
  ];

  for (const fileName of schemaFiles) {
    const schemaName = fileName.slice(0, -schemaSuffix.length);
    const schemaPath = join(schemaDir, fileName);
    const modulePath = join(outputDir, `${schemaName}.ts`);
    const schema = JSON.parse(await readFile(schemaPath, "utf8"));
    const dereferencedSchema = await $RefParser.dereference(schema);

    const constName = `${toCamelCase(schemaName)}GeneratedSchema`;
    const typeName = `${schemaName}FromSchema`;
    const expression = jsonSchemaToZod(dereferencedSchema, {
      module: "none",
      parserOverride
    }).trim();

    const moduleSource = [
      "// Generated file, update with `bun run contracts:gen`.",
      "import { z } from \"zod\";",
      "",
      `export const ${constName} = ${expression};`,
      `export type ${typeName} = z.infer<typeof ${constName}>;`,
      ""
    ].join("\n");

    await writeFile(modulePath, moduleSource, "utf8");

    indexLines.push(`export { ${constName} } from "./${schemaName}";`);
    indexLines.push(`export type { ${typeName} } from "./${schemaName}";`);
  }

  indexLines.push("");
  await writeFile(join(outputDir, "index.ts"), indexLines.join("\n"), "utf8");
}

run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

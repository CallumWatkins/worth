import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import $RefParser from "@apidevtools/json-schema-ref-parser";
import { jsonSchemaToZod } from "json-schema-to-zod";

const schemaDir = join(process.cwd(), "app", "generated", "schemas");
const outputDir = join(process.cwd(), "app", "generated", "zod");

const schemaSuffix = ".schema.json";

function toCamelCase(name) {
  if (!name.length) return name;
  return `${name[0].toLowerCase()}${name.slice(1)}`;
}

function isNullableDateSchema(schema) {
  if (!schema || typeof schema !== "object") return false;
  if (schema.format !== "date") return false;
  return Array.isArray(schema.type) && schema.type.includes("null") && schema.type.includes("string");
}

function parserOverride(schema) {
  if (isNullableDateSchema(schema)) {
    return "z.preprocess((value) => value === \"\" ? null : value, z.union([z.string().date(), z.null()]).default(null))";
  }

  if (isConstOneOfSchema(schema)) {
    return `z.union([${schema.oneOf.map(oneOfConstToLiteral).join(", ")}])`;
  }
}

function isRecord(value) {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function shouldUseKindDiscriminator(schema) {
  if (!isRecord(schema) || !Array.isArray(schema.oneOf) || schema.oneOf.length < 2) return false;

  return schema.oneOf.every((variant) => {
    if (!isRecord(variant) || variant.type !== "object" || !isRecord(variant.properties)) return false;
    const kind = variant.properties.kind;
    if (!isRecord(kind) || kind.type !== "string" || !Array.isArray(kind.enum)) return false;
    if (kind.enum.length !== 1 || typeof kind.enum[0] !== "string") return false;
    if (!Array.isArray(variant.required) || !variant.required.includes("kind")) return false;
    return true;
  });
}

function isConstOneOfSchema(schema) {
  if (!isRecord(schema) || !Array.isArray(schema.oneOf) || !schema.oneOf.length) return false;
  return schema.oneOf.every((variant) => {
    if (!isRecord(variant)) return false;
    const keys = Object.keys(variant);
    return keys.length === 1 && keys[0] === "const";
  });
}

function oneOfConstToLiteral(variant) {
  const value = variant.const;
  return `z.literal(${JSON.stringify(value)})`;
}

function annotateDiscriminators(schema) {
  if (!isRecord(schema)) return;

  if (shouldUseKindDiscriminator(schema)) {
    schema.discriminator = { propertyName: "kind" };
  }

  if (Array.isArray(schema.oneOf)) {
    for (const sub of schema.oneOf) annotateDiscriminators(sub);
  }
  if (Array.isArray(schema.anyOf)) {
    for (const sub of schema.anyOf) annotateDiscriminators(sub);
  }
  if (Array.isArray(schema.allOf)) {
    for (const sub of schema.allOf) annotateDiscriminators(sub);
  }

  if (isRecord(schema.items)) annotateDiscriminators(schema.items);
  if (Array.isArray(schema.items)) {
    for (const sub of schema.items) annotateDiscriminators(sub);
  }

  if (isRecord(schema.properties)) {
    for (const value of Object.values(schema.properties)) {
      annotateDiscriminators(value);
    }
  }
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
    annotateDiscriminators(dereferencedSchema);

    const constName = `${toCamelCase(schemaName)}GeneratedSchema`;
    const typeName = `${schemaName}FromSchema`;
    const expression = jsonSchemaToZod(dereferencedSchema, { module: "none", parserOverride })
      .trim()
      .replace(/"([a-z_]\w*)":/gi, "$1:")
      .replace(/,(?=\S)/g, ", ");

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

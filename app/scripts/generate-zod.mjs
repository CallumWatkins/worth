import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import $RefParser from "@apidevtools/json-schema-ref-parser";
import { jsonSchemaToZod, parseSchema } from "json-schema-to-zod";

const schemaDir = join(process.cwd(), "app", "generated", "schemas");
const outputDir = join(process.cwd(), "app", "generated", "zod");

const schemaSuffix = ".schema.json";

const calendarDateSchema = "z.custom<import(\"@internationalized/date\").CalendarDate>((value) => value != null && typeof value === \"object\" && typeof value.toString === \"function\")";

function toCamelCase(name) {
  if (!name.length) return name;
  return `${name[0].toLowerCase()}${name.slice(1)}`;
}

function isDateSchemaNode(schemaNode) {
  if (!schemaNode || typeof schemaNode !== "object" || schemaNode.format !== "date") return false;

  const type = schemaNode.type;
  if (type === "string") return true;

  if (Array.isArray(type)) {
    const types = new Set(type);
    return types.has("string") && [...types].every((item) => item === "string" || item === "null");
  }

  return false;
}

function isNullableDateSchemaNode(schemaNode) {
  if (!isDateSchemaNode(schemaNode)) return false;
  if (schemaNode.nullable === true) return true;
  return Array.isArray(schemaNode.type) && schemaNode.type.includes("null");
}

function applyNodeMetadata(expression, schemaNode, refs) {
  let out = expression;

  if (!refs?.withoutDescribes && schemaNode.description) {
    out += `.describe(${JSON.stringify(schemaNode.description)})`;
  }

  if (!refs?.withoutDefaults && schemaNode.default !== undefined) {
    out += `.default(${JSON.stringify(schemaNode.default)})`;
  }

  if (schemaNode.readOnly) {
    out += ".readonly()";
  }

  return out;
}

function dateParserOverride(schemaNode, refs) {
  if (!isDateSchemaNode(schemaNode)) return;

  const innerSchema = parseSchema(
    schemaNode,
    {
      ...refs,
      parserOverride: undefined,
      withoutDefaults: true,
      withoutDescribes: true,
      seen: new Map()
    },
    true
  );

  const isNullable = isNullableDateSchemaNode(schemaNode);
  const wrapper = isNullable
    ? `z.union([z.null(), ${calendarDateSchema}]).transform((value) => value == null ? null : value.toString()).pipe(${innerSchema}).transform((value) => value ?? undefined)`
    : `${calendarDateSchema}.transform((value) => value.toString()).pipe(${innerSchema})`;

  return applyNodeMetadata(wrapper, schemaNode, refs);
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
      parserOverride: dateParserOverride
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

import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import $RefParser from "@apidevtools/json-schema-ref-parser";
import { jsonSchemaToZod, parseSchema } from "json-schema-to-zod";

const schemaDir = join(process.cwd(), "app", "generated", "schemas");
const outputDir = join(process.cwd(), "app", "generated", "zod");

const schemaSuffix = ".schema.json";

const calendarDateSchema = "z.custom<import(\"@internationalized/date\").CalendarDate>((value) => value != null && typeof value === \"object\" && typeof value.toString === \"function\")";
const validationExtensionKey = "x-validation";

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

function validationMessages(schemaNode) {
  const value = schemaNode?.[validationExtensionKey];
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  return value;
}

function zodErrorObject(message) {
  return `{ error: ${JSON.stringify(message)} }`;
}

function zodTypeErrorParam(messages) {
  const required = messages.required;
  const invalid = messages.type ?? messages.invalid ?? required;

  if (required && invalid && required !== invalid) {
    return `{ error: (issue) => issue.input === undefined ? ${JSON.stringify(required)} : ${JSON.stringify(invalid)} }`;
  }

  if (invalid) return zodErrorObject(invalid);
  return undefined;
}

function replaceFirstCall(expression, call, replacement) {
  return expression.startsWith(call)
    ? `${replacement}${expression.slice(call.length)}`
    : expression;
}

function addArgumentToInitialCall(expression, callee, argument) {
  const callStart = `${callee}(`;
  if (!expression.startsWith(callStart)) return expression;

  let depth = 0;
  let quote = null;
  let escaped = false;

  for (let index = callee.length; index < expression.length; index += 1) {
    const char = expression[index];

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === "\"" || char === "'" || char === "`") {
      quote = char;
      continue;
    }

    if (char === "(") {
      depth += 1;
      continue;
    }

    if (char !== ")") continue;

    depth -= 1;
    if (depth === 0) {
      return `${expression.slice(0, index)}, ${argument}${expression.slice(index)}`;
    }
  }

  return expression;
}

function applyValidationMessages(expression, schemaNode) {
  const messages = validationMessages(schemaNode);
  if (!messages) return expression;

  let out = expression;
  const schemaTypes = Array.isArray(schemaNode.type) ? schemaNode.type : [schemaNode.type];
  const isArraySchema = schemaTypes.includes("array");
  const isStringSchema = schemaTypes.includes("string");
  const typeErrorParam = zodTypeErrorParam(messages);

  if (typeErrorParam) {
    out = replaceFirstCall(out, "z.string()", `z.string(${typeErrorParam})`);
    out = replaceFirstCall(out, "z.number()", `z.number(${typeErrorParam})`);
    out = addArgumentToInitialCall(out, "z.array", typeErrorParam);
    out = addArgumentToInitialCall(out, "z.enum", typeErrorParam);
    out = addArgumentToInitialCall(out, "z.literal", typeErrorParam);
    out = addArgumentToInitialCall(out, "z.union", typeErrorParam);
    out = addArgumentToInitialCall(out, "z.discriminatedUnion", typeErrorParam);
  }

  if (messages.blank || messages.pattern) {
    out = out.replace(/\.regex\((new RegExp\([^)]*\))\)/g, `.regex($1, ${zodErrorObject(messages.blank ?? messages.pattern)})`);
  }

  if (isStringSchema && (messages.minLength || messages.blank || messages.required)) {
    out = out.replace(/\.min\((\d+)\)/g, `.min($1, ${zodErrorObject(messages.minLength ?? messages.blank ?? messages.required)})`);
  }

  if (isStringSchema && messages.maxLength) {
    out = out.replace(/\.max\((\d+)\)/g, `.max($1, ${zodErrorObject(messages.maxLength)})`);
  }

  if (isArraySchema && (messages.minItems || messages.required)) {
    out = out.replace(/\.min\((\d+)\)/g, `.min($1, ${zodErrorObject(messages.minItems ?? messages.required)})`);
  }

  if (messages.minimum) {
    out = out.replace(/\.gte\((-?\d+)\)/g, `.gte($1, ${zodErrorObject(messages.minimum)})`);
  }

  return out;
}

function validationMessagesParserOverride(schemaNode, refs) {
  if (!validationMessages(schemaNode)) return;

  const schemaWithoutOwnValidation = { ...schemaNode };
  delete schemaWithoutOwnValidation[validationExtensionKey];

  const expression = parseSchema(
    schemaWithoutOwnValidation,
    {
      ...refs,
      parserOverride,
      withoutDefaults: true,
      withoutDescribes: true,
      seen: new Map()
    },
    true
  );

  return applyNodeMetadata(applyValidationMessages(expression, schemaNode), schemaNode, refs);
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

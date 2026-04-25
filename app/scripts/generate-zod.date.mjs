// Converts Rust date contract fields into frontend-friendly Zod schemas.
// Forms use @internationalized/date CalendarDate objects, while IPC contracts expect ISO strings,
// so this override validates CalendarDate input and then pipes it through the generated string date schema.

import { parseSchema } from "json-schema-to-zod";

import { applyNodeMetadata } from "./generate-zod.metadata.mjs";

const calendarDateSchema = "z.custom<import(\"@internationalized/date\").CalendarDate>((value) => value != null && typeof value === \"object\" && typeof value.toString === \"function\")";

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

export function dateParserOverride(schemaNode, refs) {
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

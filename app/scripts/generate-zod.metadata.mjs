// Shared helpers for preserving JSON Schema metadata when overriding json-schema-to-zod parsers.
// Parser overrides replace the library's default expression, so any supported metadata must be
// explicitly re-applied here to keep generated Zod schemas equivalent to the default conversion.

export function applyNodeMetadata(expression, schemaNode, refs) {
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

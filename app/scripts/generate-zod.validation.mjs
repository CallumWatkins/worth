// Translates backend-owned `x-validation` JSON Schema metadata into Zod error parameters.
// The Rust contracts remain the SSoT for message copy; this module only teaches the generator
// how to attach that copy to the Zod expressions emitted by json-schema-to-zod.

import { parseSchema } from "json-schema-to-zod";

import { applyNodeMetadata } from "./generate-zod.metadata.mjs";

const validationExtensionKey = "x-validation";

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

// Adds a final argument to the first call in an emitted Zod expression, e.g. z.array(inner)
// becomes z.array(inner, { error: "..." }). This avoids brittle regex matching for nested calls.
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

export function createValidationMessagesParserOverride(getParserOverride) {
  return function validationMessagesParserOverride(schemaNode, refs) {
    if (!validationMessages(schemaNode)) return;

    const schemaWithoutOwnValidation = { ...schemaNode };
    delete schemaWithoutOwnValidation[validationExtensionKey];

    const expression = parseSchema(
      schemaWithoutOwnValidation,
      {
        ...refs,
        parserOverride: getParserOverride(),
        withoutDefaults: true,
        withoutDescribes: true,
        seen: new Map()
      },
      true
    );

    return applyNodeMetadata(applyValidationMessages(expression, schemaNode), schemaNode, refs);
  };
}

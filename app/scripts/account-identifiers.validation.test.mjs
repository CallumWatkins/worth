import assert from "node:assert/strict";
// eslint-disable-next-line test/no-import-node-test -- These standalone script tests intentionally use only the Node.js test runner.
import test from "node:test";
import { accountUpsertInputGeneratedSchema } from "../generated/zod/AccountUpsertInput.ts";

const identifiersSchema = accountUpsertInputGeneratedSchema.shape.identifiers;

test("account details are optional and preserve arbitrary international labels and duplicate entries", () => {
  assert.deepEqual(identifiersSchema.parse([]), []);
  const identifiers = [
    { label: "支店コード", value: "００１２３" },
    { label: "رقم الحساب", value: "0012-3456" },
    { label: "Référence contrat", value: " AbC / 00-12 " },
    { label: "Référence contrat", value: " AbC / 00-12 " }
  ];
  assert.deepEqual(identifiersSchema.parse(identifiers), identifiers);
});

test("each detail requires a label and a string value, with nested field paths", () => {
  for (const [field, message] of [["label", "Enter an identifier"], ["value", "Value must be text"]]) {
    const identifier = { label: "IBAN", value: "DE00 0000" };
    delete identifier[field];
    const result = identifiersSchema.safeParse([identifier]);
    assert.equal(result.success, false);
    assert.deepEqual(result.error.issues[0].path, [0, field]);
    assert.equal(result.error.issues[0].message, message);
  }
});

test("values can be empty or spaces while identifiers remain required", () => {
  for (const value of ["", " ", "\u00A0\u3000"]) {
    const identifier = { label: "IBAN", value };
    assert.deepEqual(identifiersSchema.parse([identifier]), [identifier]);
    const result = identifiersSchema.safeParse([{ label: value, value: "" }]);
    assert.equal(result.success, false);
    assert.deepEqual(result.error.issues[0].path, [0, "label"]);
    assert.equal(result.error.issues[0].message, "Enter an identifier");
  }
});

test("multiline and control characters are rejected in labels and values", () => {
  const invalidValues = [
    "\t",
    "\n",
    "\r\n",
    "first\nsecond",
    "number\n",
    "number\r",
    "number\r\n",
    "\u0000number",
    "number\t",
    "number\u007F",
    "number\u0085",
    "number\u2028",
    "number\u2029"
  ];
  for (const field of ["label", "value"]) {
    for (const value of invalidValues) {
      const result = identifiersSchema.safeParse([{ label: "Reference", value: "000123", [field]: value }]);
      assert.equal(result.success, false, `${field} must reject ${JSON.stringify(value)}`);
      assert.deepEqual(result.error.issues[0].path, [0, field]);
      assert.equal(result.error.issues[0].message, field === "label" && value.trim() === ""
        ? "Enter an identifier"
        : `${field === "label" ? "Identifier" : "Value"} must be on one line without control characters`);
    }
  }
});

test("label and value limits count Unicode code points rather than UTF-16 units", () => {
  assert.equal(identifiersSchema.safeParse([{ label: "𠮷".repeat(80), value: "𠮷".repeat(250) }]).success, true);
  for (const [field, length, message] of [
    ["label", 81, "Identifier must be 80 characters or fewer"],
    ["value", 251, "Value must be 250 characters or fewer"]
  ]) {
    const result = identifiersSchema.safeParse([{ label: "Reference", value: "0012", [field]: "𠮷".repeat(length) }]);
    assert.equal(result.success, false);
    assert.deepEqual(result.error.issues[0].path, [0, field]);
    assert.equal(result.error.issues[0].message, message);
  }
});

test("accounts can have more than 50 identifiers", () => {
  const identifiers = Array.from({ length: 200 }, (_, index) => ({ label: `Reference ${index}`, value: `00${index}` }));
  assert.deepEqual(identifiersSchema.parse(identifiers), identifiers);
});

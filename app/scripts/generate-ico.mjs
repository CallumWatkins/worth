import { writeFile } from "node:fs/promises";
import { join } from "node:path";

import pngToIco from "png-to-ico";

const icoSizes = [32, 16, 24, 48, 64, 256];
const inputFiles = icoSizes.map((size) => join(process.cwd(), "brand", "tauri", "ico-src", `${size}.png`));
const outputFile = join(process.cwd(), "brand", "tauri", "icon.ico");

const ico = await pngToIco(inputFiles);

await writeFile(outputFile, ico);

console.log(`Generated ${outputFile} with entries: ${icoSizes.map((size) => `${size}x${size}`).join(", ")}`);

import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { ConversionRequest } from "./ImageConverter";

interface ImageConversionResult {
  sourceName: string;
  outputPath: string | null;
  error: string | null;
}

interface ConvertImagesResponse {
  outputDirectory: string;
  results: ImageConversionResult[];
}

export async function convertImages(
  request: ConversionRequest
): Promise<string | null> {
  try {
    const images = await Promise.all(
      request.images.map(async ({ file, rotation }) => ({
        name: file.name,
        data: new Uint8Array(await file.arrayBuffer()),
        rotation,
      }))
    );

    const response = await invoke<ConvertImagesResponse>("convert_images", {
      request: {
        images,
        targetFormat: request.targetFormat,
        ...(request.jpegQuality !== undefined
          ? { jpegQuality: request.jpegQuality }
          : {}),
        ...(request.pngCompression !== undefined
          ? { pngCompression: request.pngCompression }
          : {}),
      },
    });

    const failures = response.results.filter((r) => r.error !== null);
    const firstSuccess = response.results.find((r) => r.outputPath !== null);

    if (firstSuccess?.outputPath) {
      try {
        await revealItemInDir(firstSuccess.outputPath);
      } catch {
        void 0;
      }
    }

    if (failures.length > 0) {
      return failures
        .map((f) => `${f.sourceName}: ${f.error}`)
        .join("\n");
    }
    return null;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

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
    const images = request.images.map(({ path, rotation }) => ({ path, rotation }));

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
        ...(request.webpQuality !== undefined
          ? { webpQuality: request.webpQuality }
          : {}),
        ...(request.avifQuality !== undefined
          ? { avifQuality: request.avifQuality }
          : {}),
        ...(request.avifSpeed !== undefined
          ? { avifSpeed: request.avifSpeed }
          : {}),
        ...(request.svgPreset !== undefined
          ? { svgPreset: request.svgPreset }
          : {}),
        ...(request.svgColorMode !== undefined
          ? { svgColorMode: request.svgColorMode }
          : {}),
        ...(request.svgDetail !== undefined
          ? { svgDetail: request.svgDetail }
          : {}),
        ...(request.svgSmoothness !== undefined
          ? { svgSmoothness: request.svgSmoothness }
          : {}),
        ...(request.svgColorDetail !== undefined
          ? { svgColorDetail: request.svgColorDetail }
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

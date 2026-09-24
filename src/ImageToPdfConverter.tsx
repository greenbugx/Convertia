import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import convertiaSvg from "./assets/convertia.svg";

const IMAGE_EXTENSIONS = new Set([
  "jpeg",
  "jpg",
  "png",
  "svg",
  "webp",
  "avif",
  "tiff",
  "tif",
  "ico",
  "bmp",
]);

const VALID_EXTENSIONS = new Set([...IMAGE_EXTENSIONS, "pdf"]);

function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function isPdfPath(path: string): boolean {
  const name = fileNameOf(path);
  const dotIndex = name.lastIndexOf(".");
  if (dotIndex < 0) return false;
  return name.slice(dotIndex + 1).toLowerCase() === "pdf";
}

function isImagePath(path: string): boolean {
  const name = fileNameOf(path);
  const dotIndex = name.lastIndexOf(".");
  if (dotIndex < 0) return false;
  return IMAGE_EXTENSIONS.has(name.slice(dotIndex + 1).toLowerCase());
}

function isSupportedPath(path: string): boolean {
  return isImagePath(path) || isPdfPath(path);
}

export type PageOrientation = "Portrait" | "Landscape";
export type PageSize = "fit" | "a4" | "usletter";
export type PageMargin = "no" | "small" | "big";

export type PdfImageFormat = "PNG" | "JPEG" | "WebP" | "AVIF" | "TIFF" | "BMP";
export type PdfPagesMode = "All" | "Range";
export type PdfRotation = 0 | 90 | 180 | 270;
export type PdfBackground = "White" | "Transparent";

const RESOLUTION_PRESETS = [96, 150, 200, 300, 600] as const;
export type PdfResolution = typeof RESOLUTION_PRESETS[number];
const PDF_IMAGE_FORMATS: PdfImageFormat[] = [
  "PNG",
  "JPEG",
  "WebP",
  "AVIF",
  "TIFF",
  "BMP",
];

interface PdfFileItem {
  id: string;
  path: string;
  name: string;
}

interface ImageToPdfConverterProps {
  onConvert?: () => Promise<string | null> | void;
  isConverting?: boolean;
}

function ImageToPdfConverter({
  onConvert,
  isConverting = false,
}: ImageToPdfConverterProps = {}) {
  const [files, setFiles] = useState<PdfFileItem[]>([]);
  const [orientation, setOrientation] = useState<PageOrientation>("Portrait");
  const [pageSize, setPageSize] = useState<PageSize>("fit");
  const [margin, setMargin] = useState<PageMargin>("no");
  const [mergeAll, setMergeAll] = useState(true);
  const [pdfFormat, setPdfFormat] = useState<PdfImageFormat>("JPEG");
  const [pagesMode, setPagesMode] = useState<PdfPagesMode>("All");
  const [pageFrom, setPageFrom] = useState<number>(1);
  const [pageTo, setPageTo] = useState<number>(1);
  const [resolution, setResolution] = useState<PdfResolution>(150);
  const [pdfRotation, setPdfRotation] = useState<PdfRotation>(0);
  const [pdfBackground, setPdfBackground] = useState<PdfBackground>("White");
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const errorTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const hasImages = files.some((f) => isImagePath(f.path));
  const hasPdfs = files.some((f) => isPdfPath(f.path));
  const isPdfMode = hasPdfs && !hasImages;
  const isImageMode = hasImages && !hasPdfs;

  useEffect(() => {
    return () => {
      if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    try {
      getCurrentWebview()
        .onDragDropEvent((event) => {
          const payload = event.payload;
          if (payload.type === "enter" || payload.type === "over") {
            setIsDraggingOver(true);
          } else if (payload.type === "leave") {
            setIsDraggingOver(false);
          } else if (payload.type === "drop") {
            setIsDraggingOver(false);
            void addPaths(payload.paths);
          }
        })
        .then((fn) => {
          if (disposed) fn();
          else unlisten = fn;
        })
        .catch(() => {});
    } catch {}
    return () => {
      disposed = true;
      if (unlisten) unlisten();
    };
  }, []);

  function triggerError(msg: string) {
    setErrorMessage(msg);
    if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    errorTimerRef.current = setTimeout(() => {
      setErrorMessage(null);
    }, 6000);
  }

  async function addPaths(paths: string[]) {
    if (!paths || paths.length === 0) return;
    const supported: string[] = [];
    const unsupported: string[] = [];

    paths.forEach((p) => {
      if (isSupportedPath(p)) {
        supported.push(p);
      } else {
        unsupported.push(p);
      }
    });

    if (unsupported.length > 0) {
      const names = unsupported.map((p) => `"${fileNameOf(p)}"`).join(", ");
      triggerError(
        `Unsupported file: ${names}. Supported files: JPEG, PNG, SVG, WebP, AVIF, TIFF, ICO, BMP, PDF.`
      );
    }

    if (supported.length === 0) return;

    const incomingImages = supported.filter(isImagePath);
    const incomingPdfs = supported.filter(isPdfPath);

    const existingImages = files.filter((f) => isImagePath(f.path));
    const existingPdfs = files.filter((f) => isPdfPath(f.path));

    let toAdd: string[] = [];

    if (existingImages.length > 0) {
      if (incomingPdfs.length > 0) {
        triggerError("Cannot add PDF files while images are selected. Clear files first to convert PDFs.");
      }
      toAdd = incomingImages;
    } else if (existingPdfs.length > 0) {
      if (incomingImages.length > 0) {
        triggerError("Cannot add image files while a PDF is selected. Clear files first to convert images.");
      }
      toAdd = incomingPdfs;
    } else {
      if (incomingImages.length > 0 && incomingPdfs.length > 0) {
        triggerError("Cannot mix images and PDFs. Only images were added. Clear files first to convert PDFs.");
        toAdd = incomingImages;
      } else {
        toAdd = supported;
      }
    }

    if (toAdd.length === 0) return;

    try {
      await invoke("set_preview_paths", { paths: toAdd });
    } catch {}

    setFiles((prev) => [
      ...prev,
      ...toAdd.map((path) => ({
        id: `${path}-${Math.random().toString(36).substring(2, 9)}`,
        path,
        name: fileNameOf(path),
      })),
    ]);
  }

  async function browseFiles() {
    const existingImages = files.filter((f) => isImagePath(f.path));
    const existingPdfs = files.filter((f) => isPdfPath(f.path));

    const filters =
      existingImages.length > 0
        ? [{ name: "Images", extensions: [...IMAGE_EXTENSIONS] }]
        : existingPdfs.length > 0
        ? [{ name: "PDF Files", extensions: ["pdf"] }]
        : [{ name: "Images & PDFs", extensions: [...VALID_EXTENSIONS] }];

    try {
      const selected = await open({
        multiple: true,
        filters,
      });
      if (!selected) return;
      await addPaths(Array.isArray(selected) ? selected : [selected]);
    } catch {
      triggerError("Could not open the file picker.");
    }
  }

  async function handleConvert() {
    if (files.length === 0 || isConverting) return;
    if (onConvert) {
      await onConvert();
    }
  }

  return (
    <div
      className={`converter-container pdf-converter-container ${files.length === 0 ? "pdf-empty-mode" : "pdf-active-mode"}`}
    >
      <section className="converter-card preview-card pdf-preview-card">
        <div className="converter-card-header">
          <div className="header-title-group">
            <span className="card-header-title">Preview</span>
            {files.length > 0 && (
              <span className="image-counter-pill">
                {files.length} {files.length === 1 ? "file" : "files"}
              </span>
            )}
          </div>
          {files.length > 0 && (
            <button
              type="button"
              className="card-header-action-btn"
              onClick={() => setFiles([])}
              title="Clear all files"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
              <span>Clear All</span>
            </button>
          )}
        </div>

        {errorMessage && (
          <div className="converter-error-banner" role="alert">
            <div className="error-banner-content">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="error-icon">
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span className="error-text">{errorMessage}</span>
            </div>
            <button
              type="button"
              className="error-close-btn"
              onClick={() => setErrorMessage(null)}
              title="Dismiss error"
              aria-label="Dismiss error"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        )}

        <div className="preview-card-body">
          <div
            className={`dropzone-area ${isDraggingOver ? "dragging-over" : ""}`}
            onClick={() => {
              void browseFiles();
            }}
            onDragOver={(e) => {
              e.preventDefault();
              setIsDraggingOver(true);
            }}
            onDragLeave={() => setIsDraggingOver(false)}
          >
            <div className="dropzone-icon-circle">
              <svg
                width="38"
                height="38"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.7"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <polyline points="21 15 16 10 5 21" />
              </svg>
            </div>
            <h3 className="dropzone-main-text">
              {files.length > 0
                ? `${files.length} ${files.length === 1 ? "file" : "files"} selected`
                : "Select a file"}
            </h3>
            <p className="dropzone-sub-text">
              {files.length > 0
                ? "or drag & drop more images or PDFs here"
                : "or drag & drop your images or PDFs here"}
            </p>
            <div className="dropzone-formats-badge">
              Only .jpeg, .png, .svg, .webp, .avif, .tiff, .ico, .bmp, .pdf formats are supported.
            </div>
            <button
              type="button"
              className="dropzone-browse-btn"
              onClick={(e) => {
                e.stopPropagation();
                void browseFiles();
              }}
            >
              {files.length > 0 ? "Browse More Files" : "Browse Files"}
            </button>
          </div>
        </div>
      </section>

      <section
        className={`converter-card options-card pdf-options-card ${files.length === 0 ? "collapsed" : "expanded"}`}
        aria-hidden={files.length === 0}
      >
        <div className="converter-card-header">
          <h2 className="options-heading">
            {isPdfMode ? "PDF to Image Options" : "Image to PDF Options"}
          </h2>
        </div>

        <div className="options-card-body">
          {isImageMode && (
            <>
              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Page Orientation</span>
                </div>
                <div className="pdf-orientation-grid">
                  {(["Portrait", "Landscape"] as const).map((orient) => (
                    <button
                      key={orient}
                      type="button"
                      className={`pdf-chip ${orientation === orient ? "active" : ""}`}
                      onClick={() => setOrientation(orient)}
                    >
                      {orient}
                    </button>
                  ))}
                </div>
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Page Size</span>
                </div>
                <div className="pdf-select-container">
                  <select
                    className="pdf-select-dropdown"
                    value={pageSize}
                    onChange={(e) => setPageSize(e.target.value as PageSize)}
                  >
                    <option value="fit">Fit (same page size as image)</option>
                    <option value="a4">A4 (297x210mm)</option>
                    <option value="usletter">USLetter (215x274.9mm)</option>
                  </select>
                  <svg
                    className="pdf-select-arrow"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                </div>
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Margin</span>
                </div>
                <div className="pdf-margins-grid">
                  {(
                    [
                      { id: "no", label: "No margin" },
                      { id: "small", label: "Small Margin" },
                      { id: "big", label: "Big Margin" },
                    ] as const
                  ).map((item) => (
                    <button
                      key={item.id}
                      type="button"
                      className={`pdf-chip ${margin === item.id ? "active" : ""}`}
                      onClick={() => setMargin(item.id)}
                    >
                      {item.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="option-group">
                <label className="pdf-checkbox-label">
                  <input
                    type="checkbox"
                    className="pdf-checkbox-input"
                    checked={mergeAll}
                    onChange={(e) => setMergeAll(e.target.checked)}
                  />
                  <span className={`pdf-custom-checkbox ${mergeAll ? "checked" : ""}`}>
                    {mergeAll && (
                      <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="3.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    )}
                  </span>
                  <span className="pdf-checkbox-text">Merge all images as one PDF file</span>
                </label>
              </div>
            </>
          )}

          {isPdfMode && (
            <>
              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Format</span>
                </div>
                <div className="pdf-format-grid">
                  {PDF_IMAGE_FORMATS.map((fmt) => (
                    <button
                      key={fmt}
                      type="button"
                      className={`pdf-chip ${pdfFormat === fmt ? "active" : ""}`}
                      onClick={() => setPdfFormat(fmt)}
                    >
                      {fmt}
                    </button>
                  ))}
                </div>
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Pages</span>
                </div>
                <div className="pdf-orientation-grid">
                  {(["All", "Range"] as const).map((mode) => (
                    <button
                      key={mode}
                      type="button"
                      className={`pdf-chip ${pagesMode === mode ? "active" : ""}`}
                      onClick={() => setPagesMode(mode)}
                    >
                      {mode}
                    </button>
                  ))}
                </div>
                {pagesMode === "Range" && (
                  <div className="pdf-range-input-container">
                    <div className="pdf-range-field">
                      <label className="pdf-range-field-label">From</label>
                      <input
                        type="number"
                        min="1"
                        className="pdf-range-input"
                        value={pageFrom}
                        onChange={(e) => {
                          const val = parseInt(e.target.value, 10);
                          setPageFrom(isNaN(val) || val < 1 ? 1 : val);
                        }}
                      />
                    </div>
                    <span className="pdf-range-to-sep">to</span>
                    <div className="pdf-range-field">
                      <label className="pdf-range-field-label">To</label>
                      <input
                        type="number"
                        min={pageFrom}
                        className="pdf-range-input"
                        value={pageTo}
                        onChange={(e) => {
                          const val = parseInt(e.target.value, 10);
                          setPageTo(isNaN(val) || val < pageFrom ? pageFrom : val);
                        }}
                      />
                    </div>
                  </div>
                )}
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Resolution</span>
                  <span className="option-badge">{resolution} DPI</span>
                </div>
                <div className="quality-slider-container">
                  <input
                    type="range"
                    min="0"
                    max="4"
                    step="1"
                    value={RESOLUTION_PRESETS.indexOf(resolution)}
                    onChange={(e) => {
                      const idx = Number(e.target.value);
                      setResolution(RESOLUTION_PRESETS[idx]);
                    }}
                    className="quality-slider"
                  />
                </div>
                <div className="resolution-presets-row">
                  {RESOLUTION_PRESETS.map((val) => (
                    <button
                      key={val}
                      type="button"
                      className={`quality-chip ${resolution === val ? "active" : ""}`}
                      onClick={() => setResolution(val)}
                    >
                      {val}
                    </button>
                  ))}
                </div>
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Rotation</span>
                </div>
                <div className="rotate-presets-row">
                  {([0, 90, 180, 270] as const).map((deg) => (
                    <button
                      key={deg}
                      type="button"
                      className={`rotate-chip ${pdfRotation === deg ? "active" : ""}`}
                      onClick={() => setPdfRotation(deg)}
                    >
                      {deg}°
                    </button>
                  ))}
                </div>
              </div>

              <div className="option-group">
                <div className="option-group-title-row">
                  <span className="option-group-title">Background</span>
                </div>
                <div className="pdf-orientation-grid">
                  {(["White", "Transparent"] as const).map((bg) => (
                    <button
                      key={bg}
                      type="button"
                      className={`pdf-chip ${pdfBackground === bg ? "active" : ""}`}
                      onClick={() => setPdfBackground(bg)}
                    >
                      {bg}
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          <div className="convert-action-section">
            <button
              type="button"
              className={`convert-action-btn ${files.length === 0 ? "disabled" : ""} ${isConverting ? "converting" : ""}`}
              onClick={handleConvert}
              disabled={files.length === 0 || isConverting}
            >
              {isConverting ? (
                <span className="convert-btn-inner">Converting...</span>
              ) : (
                <span className="convert-btn-inner">
                  <span>C</span>
                  <img
                    src={convertiaSvg}
                    alt="o"
                    className="convert-btn-convertia-o"
                  />
                  <span>
                    {files.length > 1 ? `nvert All (${files.length})` : "nvert"}
                  </span>
                </span>
              )}
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

export default ImageToPdfConverter;

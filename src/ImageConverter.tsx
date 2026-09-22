import React, { useState, useRef, useEffect } from "react";
import convertiaSvg from "./assets/convertia.svg";

export type SupportedFormat = "jpeg" | "png" | "svg" | "webp" | "avif" | "tiff" | "ico" | "bmp";

interface FormatOption {
  id: SupportedFormat;
  label: string;
  ext: string;
  desc: string;
}

const FORMAT_OPTIONS: FormatOption[] = [
  { id: "jpeg", label: "JPEG", ext: "jpg", desc: "Standard" },
  { id: "png", label: "PNG", ext: "png", desc: "Lossless" },
  { id: "svg", label: "SVG", ext: "svg", desc: "Vector" },
  { id: "webp", label: "WebP", ext: "webp", desc: "Modern" },
  { id: "avif", label: "AVIF", ext: "avif", desc: "Next-gen" },
  { id: "tiff", label: "TIFF", ext: "tiff", desc: "High-res" },
  { id: "ico", label: "ICO", ext: "ico", desc: "Icon" },
  { id: "bmp", label: "BMP", ext: "bmp", desc: "Bitmap" },
];

const VALID_EXTENSIONS = new Set(["jpeg", "jpg", "png", "svg", "webp", "avif", "tiff", "tif", "ico", "bmp"]);

const FILE_ACCEPT_STRING = ".jpeg,.jpg,.png,.svg,.webp,.avif,.tiff,.tif,.ico,.bmp,image/jpeg,image/png,image/svg+xml,image/webp,image/avif,image/tiff,image/x-icon,image/vnd.microsoft.icon,image/bmp,image/x-ms-bmp";

function isSupportedFile(file: File): boolean {
  const ext = file.name.split(".").pop()?.toLowerCase();
  if (ext && VALID_EXTENSIONS.has(ext)) return true;
  if (
    file.type &&
    (file.type === "image/jpeg" ||
      file.type === "image/png" ||
      file.type === "image/svg+xml" ||
      file.type === "image/webp" ||
      file.type === "image/avif" ||
      file.type === "image/tiff" ||
      file.type === "image/x-icon" ||
      file.type === "image/vnd.microsoft.icon" ||
      file.type === "image/bmp" ||
      file.type === "image/x-ms-bmp")
  ) {
    return true;
  }
  return false;
}

interface ImageItem {
  id: string;
  file: File;
  url: string;
  dimensions: { width: number; height: number } | null;
  rotation: number;
}

export interface ConversionRequest {
  images: { file: File; rotation: number }[];
  targetFormat: SupportedFormat;
  jpegQuality?: number;
  pngCompression?: "Fast" | "Balanced" | "Maximum";
}

interface ImageConverterProps {
  onConvert?: (request: ConversionRequest) => Promise<string | null>;
  isConverting?: boolean;
}

export default function ImageConverter({
  onConvert,
  isConverting = false,
}: ImageConverterProps) {
  const [images, setImages] = useState<ImageItem[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [targetFormat, setTargetFormat] = useState<SupportedFormat>("png");
  const [jpegQuality, setJpegQuality] = useState(85);
  const [pngCompression, setPngCompression] = useState<"Fast" | "Balanced" | "Maximum">("Balanced");
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const isWheelLockedRef = useRef(false);
  const wheelAccumulatorRef = useRef(0);
  const wheelTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const errorTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      images.forEach((item) => URL.revokeObjectURL(item.url));
      if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    };
  }, []);

  function triggerError(msg: string) {
    setErrorMessage(msg);
    if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    errorTimerRef.current = setTimeout(() => {
      setErrorMessage(null);
    }, 6000);
  }

  function addFiles(files: FileList | File[]) {
    if (!files || files.length === 0) return;
    const fileList = Array.from(files);
    const supported: File[] = [];
    const unsupported: File[] = [];

    fileList.forEach((f) => {
      if (isSupportedFile(f)) {
        supported.push(f);
      } else {
        unsupported.push(f);
      }
    });

    if (unsupported.length > 0) {
      const names = unsupported.map((f) => `"${f.name}"`).join(", ");
      triggerError(
        `Unsupported file: ${names}. Supported files: JPEG, PNG, SVG, WebP, AVIF, TIFF, ICO, BMP.`
      );
    }

    if (supported.length === 0) return;

    const newItems: ImageItem[] = [];
    supported.forEach((file) => {
      const url = URL.createObjectURL(file);
      const id = `${file.name}-${file.lastModified}-${Math.random().toString(36).substring(2, 9)}`;
      const item: ImageItem = {
        id,
        file,
        url,
        dimensions: null,
        rotation: 0,
      };

      const img = new Image();
      img.onload = () => {
        setImages((prev) =>
          prev.map((it) =>
            it.id === id
              ? { ...it, dimensions: { width: img.naturalWidth, height: img.naturalHeight } }
              : it
          )
        );
      };
      img.src = url;

      newItems.push(item);
    });

    if (newItems.length > 0) {
      setImages((prev) => [...prev, ...newItems]);

      if (images.length === 0) {
        const ext = newItems[0].file.name.split(".").pop()?.toLowerCase();
        if (ext === "png") setTargetFormat("jpeg");
        else if (ext === "jpg" || ext === "jpeg") setTargetFormat("png");
        else if (ext === "svg") setTargetFormat("png");
        else setTargetFormat("png");
      }
    }
  }

  function removeImage(indexToRemove: number) {
    const itemToRemove = images[indexToRemove];
    if (itemToRemove) {
      URL.revokeObjectURL(itemToRemove.url);
    }
    const nextImages = images.filter((_, idx) => idx !== indexToRemove);
    setImages(nextImages);

    const nextIndex = Math.max(0, Math.min(currentIndex, nextImages.length - 1));
    setCurrentIndex(nextIndex);
  }

  function handleClearAll() {
    images.forEach((it) => URL.revokeObjectURL(it.url));
    setImages([]);
    setCurrentIndex(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  }

  function handleSetRotation(deg: number) {
    setImages((prev) =>
      prev.map((item, idx) => (idx === currentIndex ? { ...item, rotation: deg } : item))
    );
  }

  function handleRotateStep(delta: number) {
    setImages((prev) =>
      prev.map((item, idx) =>
        idx === currentIndex
          ? { ...item, rotation: (item.rotation + delta + 360) % 360 }
          : item
      )
    );
  }

  function handleWheel(e: React.WheelEvent<HTMLDivElement>) {
    if (images.length <= 1) return;

    wheelAccumulatorRef.current += e.deltaY;

    if (wheelTimerRef.current) {
      clearTimeout(wheelTimerRef.current);
    }
    wheelTimerRef.current = setTimeout(() => {
      wheelAccumulatorRef.current = 0;
    }, 160);

    if (isWheelLockedRef.current) return;

    if (Math.abs(wheelAccumulatorRef.current) >= 28) {
      const direction = wheelAccumulatorRef.current > 0 ? 1 : -1;
      const nextIndex = Math.min(Math.max(currentIndex + direction, 0), images.length - 1);

      if (nextIndex !== currentIndex) {
        isWheelLockedRef.current = true;
        wheelAccumulatorRef.current = 0;
        setCurrentIndex(nextIndex);

        setTimeout(() => {
          isWheelLockedRef.current = false;
        }, 360);
      } else {
        wheelAccumulatorRef.current = 0;
      }
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (images.length <= 1) return;
    if (e.key === "ArrowDown" || e.key === "PageDown") {
      e.preventDefault();
      setCurrentIndex((p) => Math.min(images.length - 1, p + 1));
    } else if (e.key === "ArrowUp" || e.key === "PageUp") {
      e.preventDefault();
      setCurrentIndex((p) => Math.max(0, p - 1));
    }
  }

  async function handleConvert() {
    if (images.length === 0 || isConverting) return;
    if (onConvert) {
      const error = await onConvert({
        images: images.map((it) => ({ file: it.file, rotation: it.rotation })),
        targetFormat,
        ...(targetFormat === "jpeg" ? { jpegQuality } : {}),
        ...(targetFormat === "png" ? { pngCompression } : {}),
      });
      if (error) {
        triggerError(error);
      }
    }
  }

  const currentItem = images[currentIndex];
  const currentRotation = currentItem?.rotation ?? 0;

  return (
    <div className="converter-container">
      {/* Preview of Selected Files */}
      <section className="converter-card preview-card">
        <div className="converter-card-header">
          <div className="header-title-group">
            <span className="card-header-title">Image Preview</span>
            {images.length > 0 && (
              <span className="image-counter-pill">
                {currentIndex + 1} / {images.length}
              </span>
            )}
          </div>
          {images.length > 0 && (
            <button
              type="button"
              className="card-header-action-btn"
              onClick={handleClearAll}
              title="Clear all pictures"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
              <span>Clear All</span>
            </button>
          )}
        </div>

        {/* Error Notification Banner */}
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
          {images.length === 0 ? (
            <div
              className={`dropzone-area ${isDraggingOver ? "dragging-over" : ""}`}
              onDragOver={(e) => {
                e.preventDefault();
                setIsDraggingOver(true);
              }}
              onDragLeave={() => setIsDraggingOver(false)}
              onDrop={(e) => {
                e.preventDefault();
                setIsDraggingOver(false);
                if (e.dataTransfer.files) addFiles(e.dataTransfer.files);
              }}
              onClick={() => fileInputRef.current?.click()}
            >
              <input
                type="file"
                ref={fileInputRef}
                style={{ display: "none" }}
                multiple
                accept={FILE_ACCEPT_STRING}
                onChange={(e) => {
                  if (e.target.files) addFiles(e.target.files);
                  e.target.value = "";
                }}
              />
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
              <h3 className="dropzone-main-text">Select a file</h3>
              <p className="dropzone-sub-text">or drag & drop your images here</p>
              <div className="dropzone-formats-badge">
                Only .jpeg, .png, .svg, .webp, .avif, .tiff, .ico, .bmp formats are supported.
              </div>
              <button
                type="button"
                className="dropzone-browse-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  fileInputRef.current?.click();
                }}
              >
                Browse Images
              </button>
            </div>
          ) : (
            <div
              className={`preview-active-layout ${isDraggingOver ? "dragging-over" : ""}`}
              onDragOver={(e) => {
                e.preventDefault();
                setIsDraggingOver(true);
              }}
              onDragLeave={() => setIsDraggingOver(false)}
              onDrop={(e) => {
                e.preventDefault();
                setIsDraggingOver(false);
                if (e.dataTransfer.files) addFiles(e.dataTransfer.files);
              }}
            >
              <div
                className="picture-scroll-outer"
                onWheel={handleWheel}
                tabIndex={0}
                onKeyDown={handleKeyDown}
                role="region"
                aria-label="Image Preview Carousel"
              >
                <div
                  className="picture-slider-track"
                  style={{
                    transform: `translateY(calc(-${currentIndex} * (100% + 14px)))`,
                  }}
                >
                  {images.map((item, index) => (
                    <div className="picture-box" key={item.id}>
                      <div className="picture-box-tag">
                        <span className="picture-box-number">#{index + 1}</span>
                        <span className="picture-box-name" title={item.file.name}>
                          {item.file.name}
                        </span>
                      </div>
                      <button
                        type="button"
                        className="picture-box-remove-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeImage(index);
                        }}
                        title="Remove this picture"
                      >
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <line x1="18" y1="6" x2="6" y2="18" />
                          <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      </button>
                      <img
                        src={item.url}
                        alt={item.file.name}
                        className="preview-img-element"
                        style={{
                          transform: `rotate(${item.rotation}deg)`,
                        }}
                      />
                    </div>
                  ))}
                </div>

                {images.length > 1 && (
                  <div className="picture-floating-nav">
                    <button
                      type="button"
                      className="picture-nav-btn"
                      disabled={currentIndex === 0}
                      onClick={() => setCurrentIndex((p) => Math.max(0, p - 1))}
                      title="Previous picture"
                      aria-label="Previous picture"
                    >
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="18 15 12 9 6 15" />
                      </svg>
                    </button>
                    <button
                      type="button"
                      className="picture-nav-btn"
                      disabled={currentIndex === images.length - 1}
                      onClick={() => setCurrentIndex((p) => Math.min(images.length - 1, p + 1))}
                      title="Next picture"
                      aria-label="Next picture"
                    >
                      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="6 9 12 15 18 9" />
                      </svg>
                    </button>
                  </div>
                )}
              </div>

              {/* Plus button */}
              <div className="preview-bottom-bar">
                <button
                  type="button"
                  className="add-more-pics-btn"
                  onClick={() => fileInputRef.current?.click()}
                  title="Add more pictures"
                  aria-label="Add more pictures"
                >
                  <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                </button>
                <input
                  type="file"
                  ref={fileInputRef}
                  style={{ display: "none" }}
                  multiple
                  accept={FILE_ACCEPT_STRING}
                  onChange={(e) => {
                    if (e.target.files) addFiles(e.target.files);
                    e.target.value = "";
                  }}
                />
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Image to Image Options */}
      <section className="converter-card options-card">
        <div className="converter-card-header">
          <h2 className="options-heading">Image to Image Options</h2>
        </div>

        <div className="options-card-body">
          {/* Image Rotate Options */}
          <div className="option-group">
            <div className="option-group-title-row">
              <span className="option-group-title">Image Rotate Options</span>
            </div>

            <div className="rotate-presets-row">
              {[0, 90, 180, 270].map((deg) => (
                <button
                  key={deg}
                  type="button"
                  className={`rotate-chip ${currentRotation === deg ? "active" : ""}`}
                  onClick={() => handleSetRotation(deg)}
                  disabled={images.length === 0}
                >
                  {deg}°
                </button>
              ))}
            </div>

            <div className="rotate-step-row">
              <button
                type="button"
                className="rotate-step-btn"
                onClick={() => handleRotateStep(-90)}
                disabled={images.length === 0}
                title="Rotate 90° counter-clockwise"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="1 4 1 10 7 10" />
                  <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
                </svg>
                <span>Rotate Left</span>
              </button>
              <button
                type="button"
                className="rotate-step-btn"
                onClick={() => handleRotateStep(90)}
                disabled={images.length === 0}
                title="Rotate 90° clockwise"
              >
                <span>Rotate Right</span>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="23 4 23 10 17 10" />
                  <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                </svg>
              </button>
            </div>
          </div>

          {/* Format Options */}
          <div className="option-group">
            <div className="option-group-title-row">
              <span className="option-group-title">Format Options</span>
            </div>

            <div className="format-chips-grid">
              {FORMAT_OPTIONS.map((fmt) => (
                <button
                  key={fmt.id}
                  type="button"
                  className={`format-chip ${targetFormat === fmt.id ? "active" : ""}`}
                  onClick={() => setTargetFormat(fmt.id)}
                >
                  <span className="format-name">{fmt.label}</span>
                  <span className="format-description">{fmt.desc}</span>
                </button>
              ))}
            </div>
          </div>

          {targetFormat === "jpeg" && (
            <div className="option-group">
              <div className="option-group-title-row">
                <span className="option-group-title">JPEG Quality</span>
                <span className="option-badge">{jpegQuality}%</span>
              </div>
              <div className="quality-slider-container">
                <input
                  type="range"
                  min="1"
                  max="100"
                  value={jpegQuality}
                  onChange={(e) => setJpegQuality(Number(e.target.value))}
                  className="quality-slider"
                />
              </div>
              <div className="quality-presets-row">
                {[60, 75, 85, 100].map((val) => (
                  <button
                    key={val}
                    type="button"
                    className={`quality-chip ${jpegQuality === val ? "active" : ""}`}
                    onClick={() => setJpegQuality(val)}
                  >
                    {val}%
                  </button>
                ))}
              </div>
            </div>
          )}

          {targetFormat === "png" && (
            <div className="option-group">
              <div className="option-group-title-row">
                <span className="option-group-title">PNG Compression</span>
              </div>
              <div className="compression-options-row">
                {(["Fast", "Balanced", "Maximum"] as const).map((level) => (
                  <button
                    key={level}
                    type="button"
                    className={`compression-chip ${pngCompression === level ? "active" : ""}`}
                    onClick={() => setPngCompression(level)}
                  >
                    {level}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Convert Button */}
          <div className="convert-action-section">
            <button
              type="button"
              className={`convert-action-btn ${images.length === 0 ? "disabled" : ""} ${isConverting ? "converting" : ""}`}
              onClick={handleConvert}
              disabled={images.length === 0 || isConverting}
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
                    {images.length > 1 ? `nvert All (${images.length})` : "nvert"}
                  </span>
                </span>
              )}
            </button>
            {images.length === 0 && (
              <p className="convert-helper-text">
                Select pictures on the left to start converting
              </p>
            )}
          </div>
        </div>
      </section>
    </div>
  );
}

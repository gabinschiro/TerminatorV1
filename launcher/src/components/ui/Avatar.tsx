import { useEffect, useRef } from "react";

interface AvatarProps {
  skinUrl: string;
  size?: number;
  className?: string;
}

// La tête d'un skin 64x64 occupe le carré (8,8) de côté 8.
const HEAD_X = 8;
const HEAD_Y = 8;
const HEAD_SIZE = 8;

export function Avatar({ skinUrl, size = 48, className = "" }: AvatarProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => {
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.clearRect(0, 0, size, size);
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(
        image,
        HEAD_X,
        HEAD_Y,
        HEAD_SIZE,
        HEAD_SIZE,
        0,
        0,
        size,
        size,
      );
    };
    image.src = skinUrl;
  }, [skinUrl, size]);

  return (
    <canvas
      ref={canvasRef}
      width={size}
      height={size}
      className={`border border-border ${className}`}
      aria-label="Tête du skin"
    />
  );
}
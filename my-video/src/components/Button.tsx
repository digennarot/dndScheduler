import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";

export const Button: React.FC<{
    label: string;
    isPrimary?: boolean;
    pulseColor?: string;
    delayStart?: number;
}> = ({ label, isPrimary = false, pulseColor = "#dc2626", delayStart = 0 }) => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    const pulseSpring = spring({
        frame: Math.max(0, frame - delayStart),
        fps,
        config: { damping: 12 },
    });

    const baseColor = isPrimary ? "#b91c1c" : "transparent";
    const finalColor = frame >= delayStart ? pulseColor : baseColor;

    return (
        <div
            style={{
                backgroundColor: finalColor,
                color: "#ffffff",
                padding: "24px 64px",
                borderRadius: "16px",
                fontFamily: "Inter, sans-serif",
                fontSize: "32px",
                fontWeight: "bold",
                border: isPrimary ? "none" : "2px solid #b91c1c",
                boxShadow: frame >= delayStart ? `0 0 ${20 * pulseSpring}px ${pulseColor}` : "none",
                transform: `scale(${1 + 0.05 * pulseSpring})`,
                transition: "background-color 0.2s ease",
            }}
        >
            {label}
        </div>
    );
};

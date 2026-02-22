import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate, spring, useVideoConfig } from "remotion";

export const Scene2: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    const logoOpacity = interpolate(frame, [15, 45], [0, 1], {
        extrapolateRight: "clamp",
    });

    const textOpacity = interpolate(frame, [45, 75], [0, 1], {
        extrapolateRight: "clamp",
    });

    const logoScale = spring({
        frame: frame - 15,
        fps,
        config: { damping: 12 },
    });

    return (
        <AbsoluteFill style={{ backgroundColor: "#000000", justifyContent: "center", alignItems: "center" }}>
            {/* Simulate light leak/particles behind the logo */}
            <div
                style={{
                    position: "absolute",
                    width: "600px",
                    height: "600px",
                    background: "radial-gradient(circle, rgba(220,38,38,0.15) 0%, rgba(0,0,0,0) 70%)",
                    opacity: logoOpacity,
                    transform: `scale(${1 + Math.sin(frame / 30) * 0.1})`,
                }}
            />

            <h1
                style={{
                    color: "#c084fc",
                    fontFamily: "Cinzel, serif",
                    fontSize: "120px",
                    opacity: logoOpacity,
                    transform: `scale(${logoScale})`,
                    textShadow: "0 0 40px rgba(192,132,252,0.5)",
                    margin: 0,
                }}
            >
                cronachednd.it
            </h1>

            <h2
                style={{
                    color: "#9ca3af",
                    fontFamily: "Inter, sans-serif",
                    fontSize: "48px",
                    fontWeight: 300,
                    opacity: textOpacity,
                    marginTop: "40px",
                }}
            >
                Domina il caos. Forgia la tua leggenda.
            </h2>
        </AbsoluteFill>
    );
};

import React from "react";
import { AbsoluteFill, useCurrentFrame, interpolate } from "remotion";

export const Scene5: React.FC = () => {
    const frame = useCurrentFrame();

    const slowZoom = interpolate(frame, [0, 150], [1, 0.8], {
        extrapolateRight: "clamp",
    });

    const fadeOut = interpolate(frame, [120, 150], [1, 0], {
        extrapolateRight: "clamp",
    });

    const d20Rotation = interpolate(frame, [0, 150], [0, 360]);

    return (
        <AbsoluteFill style={{ backgroundColor: "#000000", opacity: fadeOut, transform: `scale(${slowZoom})`, justifyContent: "center", alignItems: "center" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "40px" }}>
                {/* Simple inline D20 SVG substitution */}
                <div style={{ transform: `rotate(${d20Rotation}deg)`, width: "120px", height: "120px", display: "flex", justifyContent: "center", alignItems: "center" }}>
                    <svg viewBox="0 0 100 100" fill="none" stroke="#dc2626" strokeWidth="4" xmlns="http://www.w3.org/2000/svg">
                        <polygon points="50,5 95,30 95,70 50,95 5,70 5,30" />
                        <polygon points="50,5 50,95" />
                        <line x1="5" y1="30" x2="95" y2="70" />
                        <line x1="95" y1="30" x2="5" y2="70" />
                    </svg>
                </div>

                <h1 style={{ color: "#dc2626", fontFamily: "Cinzel, serif", fontSize: "100px", margin: 0 }}>
                    cronachednd.it
                </h1>
            </div>

            <h2 style={{ color: "#f8fafc", fontFamily: "Inter, sans-serif", fontSize: "48px", marginTop: "60px", textAlign: "center" }}>
                Raduna il tuo party. Senza mal di testa.
            </h2>
            <h3 style={{ color: "#dc2626", fontFamily: "Inter, sans-serif", fontSize: "36px", marginTop: "20px" }}>
                Disponibile ora su cronachednd.it
            </h3>
        </AbsoluteFill>
    );
};

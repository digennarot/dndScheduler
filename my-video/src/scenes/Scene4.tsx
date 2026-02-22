import React from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { Avatar } from "../components/Avatar";

export const Scene4: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    const player3TurnGreen = 150;

    const stampIn = spring({
        frame: frame - (player3TurnGreen + 30),
        fps,
        config: { damping: 10, mass: 2, stiffness: 150 },
    });

    const progressBarWidth = interpolate(
        frame,
        [player3TurnGreen, player3TurnGreen + 30],
        [66, 100],
        { extrapolateRight: "clamp", extrapolateLeft: "clamp" }
    );

    return (
        <AbsoluteFill style={{ backgroundColor: "#000000", padding: "80px" }}>
            <h2 style={{ color: "#f8fafc", fontFamily: "Cinzel, serif", fontSize: "48px", textAlign: "center" }}>
                Dashboard Campagna
            </h2>

            <div style={{ display: "flex", justifyContent: "space-around", marginTop: "120px" }}>
                <Avatar initials="A" status="confirmed" />
                <Avatar initials="B" status="confirmed" />
                <Avatar initials="C" status={frame >= player3TurnGreen ? "confirmed" : "pending"} />
            </div>

            {/* Progress Bar Container */}
            <div style={{ width: "80%", height: "24px", backgroundColor: "#1a0505", margin: "100px auto", borderRadius: "12px", overflow: "hidden", border: "2px solid #450a0a" }}>
                <div style={{ width: `${progressBarWidth}%`, height: "100%", backgroundColor: "#dc2626", transition: "width 0.1s linear" }} />
            </div>

            <div style={{ textAlign: "center", marginTop: "20px", color: "#9ca3af", fontFamily: "Inter, sans-serif", fontSize: "24px" }}>
                {frame >= player3TurnGreen ? "3/3 Confermati - Quorum Raggiunto!" : "2/3 Confermati - In attesa di 1"}
            </div>

            {/* Stamp Overlay */}
            <AbsoluteFill style={{ justifyContent: "center", alignItems: "center", pointerEvents: "none" }}>
                <h1
                    style={{
                        color: "#dc2626",
                        fontFamily: "Cinzel, serif",
                        fontSize: "120px",
                        border: "8px solid #dc2626",
                        padding: "20px 60px",
                        transform: `scale(${stampIn}) rotate(-15deg)`,
                        opacity: stampIn,
                        textShadow: "0 0 20px rgba(220,38,38,0.5)",
                        boxShadow: "inset 0 0 20px rgba(220,38,38,0.5), 0 0 20px rgba(220,38,38,0.5)",
                        backgroundColor: "rgba(10,0,0,0.8)",
                    }}
                >
                    QUORUM RAGGIUNTO
                </h1>
            </AbsoluteFill>

            {/* Text Overlay */}
            <AbsoluteFill style={{ justifyContent: "flex-end", alignItems: "center", paddingBottom: "60px" }}>
                <h2 style={{ color: "#f8fafc", fontFamily: "Inter, sans-serif", fontSize: "48px", textAlign: "center" }}>
                    Logica di Quorum Automatica.<br />
                    Noi facciamo i calcoli, tu gestisci il gioco.
                </h2>
            </AbsoluteFill>
        </AbsoluteFill>
    );
};

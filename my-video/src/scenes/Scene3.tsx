import React from "react";
import { AbsoluteFill, useCurrentFrame, spring, useVideoConfig, interpolate } from "remotion";
import { Button } from "../components/Button";

export const Scene3: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    const phoneUp = spring({
        frame,
        fps,
        config: { damping: 14 },
    });

    const notificationIn = spring({
        frame: frame - 30,
        fps,
        config: { damping: 12 },
    });

    const cursorX = interpolate(frame, [120, 160], [1000, 400], {
        extrapolateRight: "clamp",
        extrapolateLeft: "clamp",
    });
    const cursorY = interpolate(frame, [120, 160], [1000, 600], {
        extrapolateRight: "clamp",
        extrapolateLeft: "clamp",
    });

    const cursorScale = interpolate(frame, [160, 165, 170], [1, 0.8, 1], {
        extrapolateRight: "clamp",
    });

    const tapFrame = 165;

    return (
        <AbsoluteFill style={{ backgroundColor: "#0a0000", justifyContent: "center", alignItems: "center" }}>
            {/* Smartphone Mockup */}
            <div
                style={{
                    width: "500px",
                    height: "900px",
                    backgroundColor: "#1a0505",
                    borderRadius: "60px",
                    border: "12px solid #450a0a",
                    transform: `translateY(${(1 - phoneUp) * 1000}px)`,
                    position: "relative",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    padding: "40px",
                    boxShadow: "0 40px 100px rgba(0,0,0,0.5)",
                }}
            >
                {/* Notification */}
                <div
                    style={{
                        width: "100%",
                        backgroundColor: "#000000",
                        borderRadius: "20px",
                        padding: "24px",
                        marginTop: "20px",
                        border: "1px solid #dc2626",
                        transform: `translateY(${(1 - notificationIn) * -200}px)`,
                        opacity: notificationIn,
                    }}
                >
                    <div style={{ color: "#dc2626", fontFamily: "Inter, sans-serif", fontSize: "20px", fontWeight: "bold", marginBottom: "8px" }}>
                        Un Drago si Avvicina...
                    </div>
                    <div style={{ color: "#f8fafc", fontFamily: "Inter, sans-serif", fontSize: "24px" }}>
                        Prossima Sessione: Sabato alle 19:00
                    </div>
                </div>

                {/* Buttons */}
                <div style={{ marginTop: "auto", display: "flex", flexDirection: "column", gap: "24px", width: "100%", opacity: notificationIn }}>
                    <div style={{ display: "flex", justifyContent: "center" }}>
                        <Button label="Pronto alla Battaglia" isPrimary={true} delayStart={tapFrame} />
                    </div>
                    <div style={{ display: "flex", justifyContent: "center" }}>
                        <Button label="Tira Iniziativa" />
                    </div>
                </div>
            </div>

            {/* Cursor */}
            <div
                style={{
                    position: "absolute",
                    left: cursorX,
                    top: cursorY,
                    width: "48px",
                    height: "48px",
                    backgroundColor: "#ffffff",
                    borderRadius: "50%",
                    boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
                    transform: `translate(-50%, -50%) scale(${cursorScale})`,
                    opacity: frame > 120 ? 1 : 0,
                }}
            />

            {/* Text Overlay */}
            <AbsoluteFill style={{ justifyContent: "center", alignItems: "flex-end", paddingRight: "100px" }}>
                <h2 style={{ color: "#f8fafc", fontFamily: "Inter, sans-serif", fontSize: "64px", maxWidth: "600px", textAlign: "right", opacity: notificationIn }}>
                    <span style={{ color: "#dc2626", fontFamily: "Cinzel, serif" }}>Magic Link</span> istantanei.<br />
                    Partecipa con un tocco.<br />
                    Nessun login richiesto.
                </h2>
            </AbsoluteFill>
        </AbsoluteFill>
    );
};

import React from "react";
import { AbsoluteFill, spring, useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import { ChatBubble } from "../components/ChatBubble";

export const Scene1: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    // Fade out everything at the end of the scene
    const opacityOut = interpolate(frame, [270, 300], [1, 0], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
    });

    // Make messages drop and fade out BEFORE title appears (frame 150)
    const messagesOpacity = interpolate(frame, [130, 150], [1, 0], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
    });

    // Spring to drop the messages down
    const messagesYOffset = spring({
        frame: Math.max(0, frame - 130),
        fps,
        config: { damping: 15 },
    });

    // Title animation: fades in and scales up
    const titleOpacity = interpolate(frame, [150, 170], [0, 1], {
        extrapolateLeft: "clamp",
        extrapolateRight: "clamp",
    });
    const titleScale = spring({
        frame: Math.max(0, frame - 150),
        fps,
        config: { damping: 12 },
    });

    const messages = [
        { text: "Giochiamo venerdì?", delay: 5, left: "2%", top: "10%", rot: -4 },
        { text: "Aspetta, ma Alex viene?", delay: 18, left: "45%", top: "20%", rot: 3 },
        { text: "Io c'ho judo...", delay: 35, left: "8%", top: "35%", rot: -2 },
        { text: "Non posso venerdì, facciamo domenica?", delay: 50, left: "40%", top: "45%", rot: 4 },
        { text: "Domenica c'ho calcetto raga", delay: 65, left: "5%", top: "60%", rot: -3 },
        { text: "Ho perso la scheda!", delay: 80, left: "50%", top: "70%", rot: 2 },
        { text: "Ragazzi?!", delay: 100, left: "25%", top: "82%", rot: 5 },
    ];

    return (
        <AbsoluteFill style={{ backgroundColor: "#0a0000", opacity: opacityOut }}>
            {/* Animated Grid Background */}
            <div style={{
                position: "absolute",
                width: "100%", height: "100%",
                backgroundSize: "100px 100px",
                backgroundImage: "linear-gradient(to right, #1a0505 2px, transparent 2px), linear-gradient(to bottom, #1a0505 2px, transparent 2px)",
                opacity: 0.8,
            }} />

            {messages.map((m, i) => {
                const scale = spring({
                    frame: frame - m.delay,
                    fps,
                    config: { damping: 10, mass: 0.8 },
                });
                return (
                    <div
                        key={i}
                        style={{
                            position: "absolute",
                            left: m.left,
                            top: `calc(${m.top} + ${messagesYOffset * 100}px)`,
                            transform: `scale(${scale}) rotate(${m.rot}deg)`,
                            opacity: messagesOpacity,
                        }}
                    >
                        <ChatBubble text={m.text} />
                    </div>
                );
            })}

            <AbsoluteFill
                style={{
                    justifyContent: "center",
                    alignItems: "center",
                    opacity: titleOpacity,
                    transform: `scale(${0.8 + titleScale * 0.2})`,
                }}
            >
                <h1
                    style={{
                        color: "#f8fafc",
                        fontFamily: "Inter, sans-serif",
                        fontWeight: 900,
                        fontSize: "60px",
                        textAlign: "center",
                        padding: "0 100px",
                        lineHeight: "1.3",
                    }}
                >
                    Organizzare la tua campagna di D&D<br />
                    non dovrebbe essere<br />
                    <span style={{
                        display: "inline-block",
                        marginTop: "30px",
                        color: "#dc2626",
                        fontFamily: "Cinzel, serif",
                        fontSize: "110px",
                        fontWeight: "bold",
                        textShadow: "0 0 50px rgba(220,38,38,0.8)",
                        transform: `scale(${1 + Math.sin(Math.max(0, frame - 150) / 10) * 0.02})`
                    }}>
                        Lo Scontro Più Difficile.
                    </span>
                </h1>
            </AbsoluteFill>
        </AbsoluteFill>
    );
};

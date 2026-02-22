import React from "react";

export const ChatBubble: React.FC<{ text: string }> = ({ text }) => {
    return (
        <div
            style={{
                backgroundColor: "#1a0505",
                color: "#f8fafc",
                padding: "16px 24px",
                borderRadius: "20px 20px 20px 4px",
                boxShadow: "0 10px 25px -5px rgba(0, 0, 0, 0.5)",
                fontFamily: "Inter, sans-serif",
                fontSize: "24px",
                border: "1px solid #450a0a",
                maxWidth: "600px",
                display: "inline-block",
            }}
        >
            {text}
        </div>
    );
};

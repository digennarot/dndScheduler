import React from "react";

export const Avatar: React.FC<{
    status?: "pending" | "confirmed";
    imageUrl?: string;
    initials?: string;
}> = ({ status = "pending", imageUrl, initials = "P" }) => {
    const borderColor = status === "confirmed" ? "#dc2626" : "#450a0a";

    return (
        <div
            style={{
                width: "120px",
                height: "120px",
                borderRadius: "50%",
                backgroundColor: "#1a0505",
                border: `6px solid ${borderColor}`,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#f8fafc",
                fontFamily: "Inter, sans-serif",
                fontSize: "48px",
                fontWeight: "bold",
                backgroundImage: imageUrl ? `url(${imageUrl})` : "none",
                backgroundSize: "cover",
                boxShadow: status === "confirmed" ? `0 0 20px ${borderColor}` : "none",
                transition: "all 0.3s ease",
            }}
        >
            {!imageUrl && initials}
        </div>
    );
};

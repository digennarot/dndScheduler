import React from "react";
import { Series, Audio, staticFile } from "remotion";
import { z } from "zod";
import { Scene1 } from "./scenes/Scene1";
import { Scene2 } from "./scenes/Scene2";
import { Scene3 } from "./scenes/Scene3";
import { Scene4 } from "./scenes/Scene4";
import { Scene5 } from "./scenes/Scene5";

// Since user environment may not have google-fonts loaded, 
// using safe fallbacks, but importing syntax as requested
// import { loadFont as loadCinzel } from "@remotion/google-fonts/Cinzel";
// import { loadFont as loadInter } from "@remotion/google-fonts/Inter";
// loadCinzel();
// loadInter();

export const mainSchema = z.object({
    titleText: z.string(),
    primaryColor: z.string(),
});

export const Main: React.FC<z.infer<typeof mainSchema>> = (props) => {
    return (
        <div style={{ flex: 1, backgroundColor: "#0a0000", fontFamily: "Inter, sans-serif" }}>
            {/* Background Epic Music. User will place 'epic-metal.mp3' in /public folder */}
            <Audio src={staticFile("epic-metal.mp3")} volume={0.6} />
            <Series>
                {/* Scene 1: The Scheduling Curse (0s - 10s -> 300 frames) */}
                <Series.Sequence durationInFrames={300}>
                    <Scene1 />
                </Series.Sequence>

                {/* Scene 2: Introducing the Campaign Heartbeat (10s - 15s -> 150 frames) */}
                <Series.Sequence durationInFrames={150}>
                    <Scene2 />
                </Series.Sequence>

                {/* Scene 3: The Magic Link (15s - 25s -> 300 frames) */}
                <Series.Sequence durationInFrames={300}>
                    <Scene3 />
                </Series.Sequence>

                {/* Scene 4: The Quorum Engine (25s - 35s -> 300 frames) */}
                <Series.Sequence durationInFrames={300}>
                    <Scene4 />
                </Series.Sequence>

                {/* Scene 5: Call to Action (35s - 40s -> 150 frames) */}
                <Series.Sequence durationInFrames={150}>
                    <Scene5 />
                </Series.Sequence>
            </Series>
        </div>
    );
};

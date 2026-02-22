import "./index.css";
import { Composition } from "remotion";
import { Main, mainSchema } from "./Main";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="DndrsLanding"
        component={Main}
        durationInFrames={1200}
        fps={30}
        width={1920}
        height={1080}
        schema={mainSchema}
        defaultProps={{
          titleText: "Organizzare la tua campagna di D&D non dovrebbe essere lo scontro più difficile.",
          primaryColor: "#dc2626",
        }}
      />
    </>
  );
};

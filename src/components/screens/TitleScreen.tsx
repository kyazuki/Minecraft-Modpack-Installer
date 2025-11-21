import DescriptionIcon from "@mui/icons-material/Description";
import {
  Box,
  Button,
  Container,
  IconButton,
  Stack,
  Typography,
} from "@mui/material";

import { invoke } from "@tauri-apps/api/core";
import { Translation } from "../../utils/localizer";
import { InstallerMode, installerModes } from "./InstallerScreen";

type TitleScreenProps = {
  onModeSelect: (mode: InstallerMode) => void;
  translation: Translation;
};

export default function TitleScreen(props: TitleScreenProps) {
  const buttonWidth = 36;
  return (
    <Container
      maxWidth="md"
      sx={{
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Stack spacing={3} alignItems="center">
        <Typography
          variant="h3"
          component="h1"
          fontWeight={700}
          textAlign="center"
          sx={{
            background: "linear-gradient(45deg, #2196F3 30%, #21CBF3 90%)",
            backgroundClip: "text",
            WebkitBackgroundClip: "text",
            WebkitTextFillColor: "transparent",
            letterSpacing: "-0.02em",
          }}
        >
          {props.translation.appTitle}
        </Typography>
        <Typography
          color="text.secondary"
          textAlign="center"
          variant="subtitle1"
        >
          {props.translation.titleMessage}
        </Typography>
        <Stack
          direction="row"
          spacing={2}
          alignItems="stretch"
          justifyContent="center"
        >
          <Box sx={{ width: buttonWidth }} />
          <Stack spacing={2} sx={{ width: "200px" }}>
            {installerModes.map((mode) => (
              <Button
                key={`title-${mode}`}
                variant="contained"
                onClick={() => props.onModeSelect(mode)}
                sx={{ height: buttonWidth }}
              >
                {props.translation.install}
              </Button>
            ))}
          </Stack>
          <IconButton
            color="primary"
            onClick={() => invoke("open_log_folder")}
            sx={{
              width: buttonWidth,
              height: buttonWidth,
              border: 1,
              borderRadius: 1,
              boxShadow: 1,
            }}
          >
            <DescriptionIcon />
          </IconButton>
        </Stack>
      </Stack>
    </Container>
  );
}

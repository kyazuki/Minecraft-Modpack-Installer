import { MenuItem, Select } from "@mui/material";
import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { Language, Translation } from "../utils/localizer";

export type TitleBarProps = {
  title: string;
  language: Language;
  onLanguageChange: (lang: Language) => void;
  translation: Translation;
};

export default function TitleBar(props: TitleBarProps) {
  return (
    <AppBar position="sticky">
      <Toolbar>
        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
          {props.title}
        </Typography>
        <Select
          size="small"
          sx={{ minWidth: 140 }}
          value={props.language}
          onChange={(event) =>
            props.onLanguageChange(event.target.value as Language)
          }
          inputProps={{
            "aria-label": props.translation.languageSelectionLabel,
          }}
        >
          <MenuItem value="ja">{props.translation.languageOptionJa}</MenuItem>
          <MenuItem value="en">{props.translation.languageOptionEn}</MenuItem>
        </Select>
      </Toolbar>
    </AppBar>
  );
}

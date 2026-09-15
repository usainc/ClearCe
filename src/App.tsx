import { t } from "./i18n";
import { Shell } from "./components/Shell";
import { Home } from "./features/home/Home";
import { Preview } from "./features/preview/Preview";
import { Batch } from "./features/batch/Batch";
import { History } from "./features/history/History";
import { Models } from "./features/models/Models";
import { Settings } from "./features/settings/Settings";
import { useApp } from "./stores/AppContext";
import { useTranslation } from "react-i18next";
export function App() {
  useTranslation();
  const { page, ready } = useApp();
  const screens = {
    Home,
    Preview,
    "Batch Queue": Batch,
    History,
    Models,
    Settings,
  };
  const Screen = screens[page];
  return (
    <Shell>
      {ready ? (
        <Screen />
      ) : (
        <div className="empty">{t("loading_your_workspace")}</div>
      )}
    </Shell>
  );
}

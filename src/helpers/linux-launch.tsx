import createBasicChoicePopup from "../Pop-Ups/Basic-Choice-PopUp/Basic-Choice-PopUp";
import createLinuxRunnerSelectionPopup from "../Pop-Ups/Linux-Runner-PopUp/Linux-Runner-PopUp";
import { LibraryApi } from "../api/library/api";
import { LaunchRunner } from "../bindings";
import { showError } from "./error";

const api = new LibraryApi();

export function isRunningOnLinux(): boolean {
  if (typeof navigator === "undefined") return false;
  return (
    /linux/i.test(navigator.userAgent) && !/android/i.test(navigator.userAgent)
  );
}

async function launchWithRunner(path: string, runner: LaunchRunner | null) {
  try {
    const result = await api.runExecutable(path, runner);
    if (result.status === "error") {
      await showError(result.error, "Failed to launch game");
    }
  } catch (err) {
    await showError(err, "Failed to launch game");
  }
}

export async function launchExecutableWithLinuxSupport(
  path: string
): Promise<void> {
  if (!isRunningOnLinux()) {
    await launchWithRunner(path, null);
    return;
  }

  let runners: LaunchRunner[] = [];
  try {
    runners = await api.getLinuxLaunchRunners();
  } catch (err) {
    await showError(err, "Failed to detect Wine/Proton runners");
    return;
  }

  if (!runners || runners.length === 0) {
    createBasicChoicePopup({
      infoTitle: "No Wine/Proton runner detected",
      infoMessage:
        "Fit Launcher could not find Wine or Proton on this system. Install Wine or a Steam Proton version, then try again.",
      confirmLabel: "OK",
      cancelLabel: "Close",
      action: async () => {},
    });
    return;
  }

  createLinuxRunnerSelectionPopup({
    runners,
    action: async (runner) => {
      await launchWithRunner(path, runner);
    },
  });
}

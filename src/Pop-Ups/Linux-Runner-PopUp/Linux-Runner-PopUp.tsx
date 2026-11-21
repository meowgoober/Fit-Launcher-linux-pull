import { For, Show, createSignal } from "solid-js";
import { render } from "solid-js/web";
import { LaunchRunner } from "../../bindings";
import { Modal } from "../Modal/Modal";

export type LinuxRunnerPopupProps = {
  infoTitle?: string;
  infoMessage?: string;
  infoFooter?: string;
  runners: LaunchRunner[];
  action: (runner: LaunchRunner) => void | Promise<void>;
  cancelAction?: () => void;
};

export default function createLinuxRunnerSelectionPopup(
  props: LinuxRunnerPopupProps
) {
  const container = document.createElement("div");
  document.body.appendChild(container);

  const [selectedId, setSelectedId] = createSignal<string>(
    props.runners[0]?.id ?? ""
  );

  const destroy = () => {
    render(() => null, container);
    container.remove();
  };

  const handleCancel = () => {
    props.cancelAction?.();
    destroy();
  };

  const handleConfirm = async () => {
    const runner = props.runners.find((r) => r.id === selectedId());
    if (!runner) return;
    await props.action(runner);
    destroy();
  };

  render(
    () => (
      <Modal
        infoTitle={props.infoTitle ?? "Select Wine/Proton Runner"}
        infoMessage={
          props.infoMessage ??
          "Choose which detected Wine/Proton installation to use for launching this game."
        }
        infoFooter={props.infoFooter}
        confirmLabel="Launch"
        cancelLabel="Cancel"
        onConfirm={handleConfirm}
        onClose={handleCancel}
        cancelAction={handleCancel}
        disabledConfirm={() => !selectedId()}
        maxWidth="lg"
      >
        <div class="flex flex-col gap-2">
          <For each={props.runners}>
            {(runner) => (
              <button
                type="button"
                class={`w-full text-left p-3 rounded-lg border transition-colors ${
                  selectedId() === runner.id
                    ? "border-accent bg-accent/10"
                    : "border-secondary-20 hover:border-accent/50 bg-background-30/40"
                }`}
                onClick={() => setSelectedId(runner.id)}
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p class="font-medium text-text truncate">{runner.name}</p>
                    <p class="text-xs text-muted truncate mt-1">{runner.path}</p>
                  </div>
                  <span class="shrink-0 text-[10px] uppercase tracking-wider px-2 py-1 rounded bg-secondary-20 text-muted border border-secondary-20">
                    {runner.runner_type}
                  </span>
                </div>
              </button>
            )}
          </For>
          <Show when={props.runners.length === 0}>
            <p class="text-sm text-muted">
              No Wine/Proton runners were detected on this system.
            </p>
          </Show>
        </div>
      </Modal>
    ),
    container
  );
}

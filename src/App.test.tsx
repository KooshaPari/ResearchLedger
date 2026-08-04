import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

const { invokeMock, openMock } = vi.hoisted(() => ({ invokeMock: vi.fn(), openMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openMock }));

function resetTauriMocks() {
  invokeMock.mockReset().mockResolvedValue({ selected: false, path: null, documentCount: 0 });
  openMock.mockReset().mockResolvedValue(null);
}

beforeEach(() => {
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: { getItem: vi.fn(() => null), setItem: vi.fn(), removeItem: vi.fn() },
  });
});

describe("ResearchLedger shell", () => {
  it("starts GitHub device polling after showing the verification code", async () => {
    resetTauriMocks();
    invokeMock.mockImplementation((command: string) => {
      if (command === "github_device_start") {
        return Promise.resolve({ deviceCode: "device", userCode: "ABCD-1234", verificationUri: "https://github.com/login/device", expiresIn: 600, interval: 5 });
      }
      if (command === "github_device_poll") return Promise.resolve("gh-token");
      return Promise.resolve({ selected: false, path: null, documentCount: 0 });
    });
    render(<App />);
    fireEvent.change(screen.getByRole("textbox", { name: "GitHub App client ID" }), { target: { value: "client-id" } });
    fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" }));
    expect(await screen.findByText("ABCD-1234")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole("button", { name: "GitHub connected" })).toBeInTheDocument());
    expect(invokeMock).toHaveBeenCalledWith("github_device_poll", expect.objectContaining({ clientId: "client-id", deviceCode: "device" }));
  });

  it("requires a GitHub client id instead of issuing a broken OAuth request", () => {
    resetTauriMocks();
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Connect GitHub" }));
    expect(screen.getByRole("status")).toHaveTextContent("Enter your GitHub App client ID");
    expect(invokeMock).not.toHaveBeenCalledWith("github_device_start", expect.anything());
  });

  it("uses a directory picker before exporting Markdown", async () => {
    resetTauriMocks();
    openMock.mockResolvedValue("/tmp/researchledger-export");
    render(<App />);
    fireEvent.change(screen.getByRole("textbox", { name: "Vault path" }), { target: { value: "/tmp/research-vault" } });
    fireEvent.click(screen.getByRole("button", { name: "Export Markdown" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("export_obsidian", { vaultPath: "/tmp/research-vault", destination: "/tmp/researchledger-export" }));
    expect(openMock).toHaveBeenCalledWith(expect.objectContaining({ directory: true, title: "Choose Markdown export folder" }));
  });

  it("shows the local-first vault setup", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "ResearchLedger" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose vault" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Capture reactions in browser" })).toBeInTheDocument();
  });

  it("opens LinkedIn sign-in with the selected persistent profile", async () => {
    resetTauriMocks();
    invokeMock.mockResolvedValue(
      "LinkedIn sign-in browser opened; close it when authentication is complete.",
    );
    render(<App />);

    fireEvent.change(screen.getByRole("textbox", { name: "LinkedIn browser profile" }), {
      target: { value: "/tmp/researchledger-linkedin" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Open LinkedIn sign-in" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("open_linkedin_signin", {
        profilePath: "/tmp/researchledger-linkedin",
      }),
    );
  });

  it("switches accessible primary workspaces", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("tab", { name: /Library/ }));
    expect(screen.getByRole("tab", { name: /Library/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Your indexed corpus");
    expect(screen.queryByRole("button", { name: "Import GitHub stars" })).not.toBeInTheDocument();
  });

  it("keeps provider form state when switching workspaces", () => {
    render(<App />);
    const profile = screen.getByRole("textbox", { name: "LinkedIn browser profile" });
    fireEvent.change(profile, { target: { value: "/tmp/researchledger-linkedin" } });
    fireEvent.click(screen.getByRole("tab", { name: /Library/ }));
    fireEvent.click(screen.getByRole("tab", { name: /Inbox/ }));
    expect(screen.getByRole("textbox", { name: "LinkedIn browser profile" })).toHaveValue("/tmp/researchledger-linkedin");
  });

  it("exposes a Hacker News saved-stories capture pathway", () => {
    render(<App />);
    expect(screen.getByText("HACKER NEWS CONNECTION")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open Hacker News sign-in" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Capture saved stories in browser" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: /Hacker News username/ })).toBeInTheDocument();
  });

  it("renders a Hacker News action card alongside LinkedIn on the source rail", () => {
    render(<App />);
    // LinkedIn + Hacker News each render a "Connect browser" action card,
    // so we use `getAllByRole` to ensure both are present.
    const connectButtons = screen.getAllByRole("button", { name: "Connect browser" });
    expect(connectButtons.length).toBeGreaterThanOrEqual(2);
    // The source rail should expose LinkedIn + Hacker News + GitHub + Enrichment actions.
    expect(screen.getByRole("button", { name: "Import starred repos" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Fetch linked sources" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Distill pending notes" })).toBeInTheDocument();
  });
});

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

describe("ResearchLedger shell", () => {
  it("shows the local-first vault setup", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "ResearchLedger" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose vault" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Capture reactions in browser" })).toBeInTheDocument();
  });

  it("switches accessible primary workspaces", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("tab", { name: /Library/ }));
    expect(screen.getByRole("tab", { name: /Library/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Your indexed corpus");
    expect(screen.queryByRole("button", { name: "Import GitHub stars" })).not.toBeInTheDocument();
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
    expect(screen.getByRole("button", { name: "Distill pending notes" })).toBeInTheDocument();
  });
});

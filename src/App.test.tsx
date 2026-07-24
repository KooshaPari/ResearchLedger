import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

describe("ResearchLedger shell", () => {
  it("shows the local-first vault setup", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "ResearchLedger" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose vault" })).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Capture in browser" })).toHaveLength(3);
    expect(screen.getByLabelText("LinkedIn capture")).toBeInTheDocument();
    expect(screen.getByLabelText("Reddit capture")).toBeInTheDocument();
    expect(screen.getByLabelText("X capture")).toBeInTheDocument();
  });

  it("switches accessible primary workspaces", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("tab", { name: /Library/ }));
    expect(screen.getByRole("tab", { name: /Library/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Your indexed corpus");
    expect(screen.queryByRole("button", { name: "Import GitHub stars" })).not.toBeInTheDocument();
  });
});

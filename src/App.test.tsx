import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

describe("ResearchLedger shell", () => {
  it("shows the local-first vault setup", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "ResearchLedger" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose local vault" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import LinkedIn HTML" })).toBeInTheDocument();
  });
});

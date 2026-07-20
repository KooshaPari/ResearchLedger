import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("ResearchLedger shell", () => {
  it("shows the local-first vault setup", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "ResearchLedger" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Select local vault" })).toBeInTheDocument();
  });
});

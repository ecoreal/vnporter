import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";
import { Button } from "@/components/ui/button";

describe("App", () => {
  it("renders the placeholder", () => {
    render(<App />);
    expect(screen.getByText(/Tauri Starter/)).toBeTruthy();
  });

  it("exposes shadcn Button", () => {
    render(<Button>click</Button>);
    expect(screen.getByRole("button", { name: "click" })).toBeTruthy();
  });
});

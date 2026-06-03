import { describe, it, expect, vi, beforeEach } from "vitest";
import { showToast, type ToastMessage, type ToastType } from "../components/Toast";

describe("showToast", () => {
  let listeners: ((toast: ToastMessage) => void)[];

  beforeEach(() => {
    // Reset global listeners
    const g = globalThis as unknown as { __toastListeners?: ((toast: ToastMessage) => void)[] };
    listeners = g.__toastListeners ??= [];
    listeners.length = 0;
  });

  it("creates toast with UUID id", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("test message", "info");

    expect(listener).toHaveBeenCalledOnce();
    const toast = listener.mock.calls[0][0];
    expect(toast.id).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/
    );
  });

  it("creates toast with correct type", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("success msg", "success");
    expect(listener.mock.calls[0][0].type).toBe("success");

    showToast("error msg", "error");
    expect(listener.mock.calls[1][0].type).toBe("error");

    showToast("info msg", "info");
    expect(listener.mock.calls[2][0].type).toBe("info");
  });

  it("defaults to info type", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("default msg");
    expect(listener.mock.calls[0][0].type).toBe("info");
  });

  it("passes message correctly", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("Hello World", "info");
    expect(listener.mock.calls[0][0].message).toBe("Hello World");
  });

  it("passes duration correctly", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("msg", "info", 5000);
    expect(listener.mock.calls[0][0].duration).toBe(5000);
  });

  it("defaults duration to 3000", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("msg");
    expect(listener.mock.calls[0][0].duration).toBe(3000);
  });

  it("notifies all listeners", () => {
    const listener1 = vi.fn();
    const listener2 = vi.fn();
    listeners.push(listener1, listener2);

    showToast("broadcast", "info");

    expect(listener1).toHaveBeenCalledOnce();
    expect(listener2).toHaveBeenCalledOnce();
  });

  it("generates unique ids for concurrent toasts", () => {
    const listener = vi.fn();
    listeners.push(listener);

    showToast("msg1", "info");
    showToast("msg2", "info");
    showToast("msg3", "info");

    const ids = listener.mock.calls.map((call) => call[0].id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(3);
  });
});

describe("ToastType", () => {
  it("accepts all valid types", () => {
    const types: ToastType[] = ["success", "error", "info"];
    types.forEach((type) => {
      const listener = vi.fn();
      const g = globalThis as unknown as { __toastListeners?: ((toast: ToastMessage) => void)[] };
      const listeners = g.__toastListeners ??= [];
      listeners.length = 0;
      listeners.push(listener);

      showToast("msg", type);
      expect(listener.mock.calls[0][0].type).toBe(type);
    });
  });
});

describe("ToastMessage interface", () => {
  it("has all required fields", () => {
    const toast: ToastMessage = {
      id: "test-id",
      type: "success",
      message: "test",
    };
    expect(toast.id).toBe("test-id");
    expect(toast.type).toBe("success");
    expect(toast.message).toBe("test");
  });

  it("has optional duration field", () => {
    const toast: ToastMessage = {
      id: "test-id",
      type: "info",
      message: "test",
      duration: 5000,
    };
    expect(toast.duration).toBe(5000);
  });
});

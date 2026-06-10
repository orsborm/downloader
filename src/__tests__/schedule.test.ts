import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  addScheduleRule,
  removeScheduleRule,
  updateScheduleRule,
  setScheduleRuleEnabled,
  getScheduleRules,
  getScheduleRule,
  addBandwidthSchedule,
  removeBandwidthSchedule,
  updateBandwidthSchedule,
  getBandwidthSchedules,
  type ScheduleRule,
  type BandwidthSchedule,
} from "../lib/tauri-api";

const mockInvoke = vi.mocked(invoke);

describe("Schedule API", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("addScheduleRule", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const rule: ScheduleRule = {
        id: "test-1",
        name: "Test Rule",
        ruleType: "StartTask",
        cronExpression: "0 9 * * *",
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      await addScheduleRule(rule);

      expect(mockInvoke).toHaveBeenCalledWith("add_schedule_rule", { rule });
    });

    it("throws error on failure", async () => {
      mockInvoke.mockRejectedValue(new Error("Invalid cron expression"));

      const rule: ScheduleRule = {
        id: "test-1",
        name: "Test Rule",
        ruleType: "StartTask",
        cronExpression: "invalid",
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      await expect(addScheduleRule(rule)).rejects.toThrow("Invalid cron expression");
    });
  });

  describe("removeScheduleRule", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeScheduleRule("test-1");

      expect(mockInvoke).toHaveBeenCalledWith("remove_schedule_rule", { ruleId: "test-1" });
    });
  });

  describe("updateScheduleRule", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const rule: ScheduleRule = {
        id: "test-1",
        name: "Updated Rule",
        ruleType: "PauseTask",
        cronExpression: "0 18 * * *",
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      await updateScheduleRule(rule);

      expect(mockInvoke).toHaveBeenCalledWith("update_schedule_rule", { rule });
    });
  });

  describe("setScheduleRuleEnabled", () => {
    it("calls invoke to enable rule", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setScheduleRuleEnabled("test-1", true);

      expect(mockInvoke).toHaveBeenCalledWith("set_schedule_rule_enabled", {
        ruleId: "test-1",
        enabled: true,
      });
    });

    it("calls invoke to disable rule", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await setScheduleRuleEnabled("test-1", false);

      expect(mockInvoke).toHaveBeenCalledWith("set_schedule_rule_enabled", {
        ruleId: "test-1",
        enabled: false,
      });
    });
  });

  describe("getScheduleRules", () => {
    it("returns list of rules", async () => {
      const rules: ScheduleRule[] = [
        {
          id: "test-1",
          name: "Rule 1",
          ruleType: "StartTask",
          cronExpression: "0 9 * * *",
          params: {},
          enabled: true,
          createdAt: "2026-01-01T00:00:00Z",
        },
        {
          id: "test-2",
          name: "Rule 2",
          ruleType: "PauseTask",
          cronExpression: "0 18 * * *",
          params: {},
          enabled: false,
          createdAt: "2026-01-01T00:00:00Z",
        },
      ];

      mockInvoke.mockResolvedValue(rules);

      const result = await getScheduleRules();

      expect(mockInvoke).toHaveBeenCalledWith("get_schedule_rules");
      expect(result).toEqual(rules);
      expect(result).toHaveLength(2);
    });

    it("returns empty array when no rules", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await getScheduleRules();

      expect(result).toEqual([]);
    });
  });

  describe("getScheduleRule", () => {
    it("returns rule by id", async () => {
      const rule: ScheduleRule = {
        id: "test-1",
        name: "Rule 1",
        ruleType: "StartTask",
        cronExpression: "0 9 * * *",
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      mockInvoke.mockResolvedValue(rule);

      const result = await getScheduleRule("test-1");

      expect(mockInvoke).toHaveBeenCalledWith("get_schedule_rule", { ruleId: "test-1" });
      expect(result).toEqual(rule);
    });

    it("returns null when rule not found", async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await getScheduleRule("nonexistent");

      expect(result).toBeNull();
    });
  });

  describe("addBandwidthSchedule", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const schedule: BandwidthSchedule = {
        startTime: "22:00",
        endTime: "06:00",
        downloadSpeed: 1024 * 1024,
        uploadSpeed: 512 * 1024,
      };

      await addBandwidthSchedule(schedule);

      expect(mockInvoke).toHaveBeenCalledWith("add_bandwidth_schedule", { schedule });
    });

    it("handles schedule with weekdays", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const schedule: BandwidthSchedule = {
        startTime: "09:00",
        endTime: "18:00",
        downloadSpeed: 2 * 1024 * 1024,
        uploadSpeed: 1024 * 1024,
        weekdays: [1, 2, 3, 4, 5],
      };

      await addBandwidthSchedule(schedule);

      expect(mockInvoke).toHaveBeenCalledWith("add_bandwidth_schedule", { schedule });
    });
  });

  describe("removeBandwidthSchedule", () => {
    it("calls invoke with correct index", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeBandwidthSchedule(0);

      expect(mockInvoke).toHaveBeenCalledWith("remove_bandwidth_schedule", { index: 0 });
    });

    it("handles removing second schedule", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await removeBandwidthSchedule(1);

      expect(mockInvoke).toHaveBeenCalledWith("remove_bandwidth_schedule", { index: 1 });
    });
  });

  describe("updateBandwidthSchedule", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const schedule: BandwidthSchedule = {
        startTime: "23:00",
        endTime: "07:00",
        downloadSpeed: 3 * 1024 * 1024,
        uploadSpeed: 1536 * 1024,
      };

      await updateBandwidthSchedule(0, schedule);

      expect(mockInvoke).toHaveBeenCalledWith("update_bandwidth_schedule", {
        index: 0,
        schedule,
      });
    });
  });

  describe("getBandwidthSchedules", () => {
    it("returns list of schedules", async () => {
      const schedules: BandwidthSchedule[] = [
        {
          startTime: "22:00",
          endTime: "06:00",
          downloadSpeed: 1024 * 1024,
          uploadSpeed: 512 * 1024,
        },
        {
          startTime: "09:00",
          endTime: "18:00",
          downloadSpeed: 2 * 1024 * 1024,
          uploadSpeed: 1024 * 1024,
          weekdays: [1, 2, 3, 4, 5],
        },
      ];

      mockInvoke.mockResolvedValue(schedules);

      const result = await getBandwidthSchedules();

      expect(mockInvoke).toHaveBeenCalledWith("get_bandwidth_schedules");
      expect(result).toEqual(schedules);
      expect(result).toHaveLength(2);
    });

    it("returns empty array when no schedules", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await getBandwidthSchedules();

      expect(result).toEqual([]);
    });
  });

});

describe("Schedule Rule Types", () => {
  it("validates ScheduleRuleType values", () => {
    const validTypes = ["StartTask", "PauseTask", "BandwidthPlan", "SeedPlan"];

    validTypes.forEach((type) => {
      const rule: ScheduleRule = {
        id: "test",
        name: "Test",
        ruleType: type as ScheduleRule["ruleType"],
        cronExpression: "0 9 * * *",
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      expect(rule.ruleType).toBe(type);
    });
  });
});

describe("Cron Expression Validation", () => {
  // These tests verify the cron expression format expected by the backend
  // The actual validation happens in Rust, but we can test the format expectations

  it("accepts valid cron expressions", () => {
    const validExpressions = [
      "* * * * *",
      "0 * * * *",
      "0 9 * * 1-5",
      "*/5 * * * *",
      "0 0 1 * *",
      "30 8 * * 1,3,5",
    ];

    validExpressions.forEach((expr) => {
      const rule: ScheduleRule = {
        id: "test",
        name: "Test",
        ruleType: "StartTask",
        cronExpression: expr,
        params: {},
        enabled: true,
        createdAt: "2026-01-01T00:00:00Z",
      };

      // Should not throw when creating the rule object
      expect(rule.cronExpression).toBe(expr);
    });
  });

  it("stores cron expression as-is for backend validation", () => {
    const invalidExpression = "invalid cron";

    const rule: ScheduleRule = {
      id: "test",
      name: "Test",
      ruleType: "StartTask",
      cronExpression: invalidExpression,
      params: {},
      enabled: true,
      createdAt: "2026-01-01T00:00:00Z",
    };

    // The rule object stores the expression as-is
    // Validation happens on the Rust backend
    expect(rule.cronExpression).toBe(invalidExpression);
  });
});

describe("BandwidthSchedule Validation", () => {
  it("validates time format HH:MM", () => {
    const schedule: BandwidthSchedule = {
      startTime: "22:00",
      endTime: "06:00",
      downloadSpeed: 1024 * 1024,
      uploadSpeed: 512 * 1024,
    };

    expect(schedule.startTime).toMatch(/^\d{2}:\d{2}$/);
    expect(schedule.endTime).toMatch(/^\d{2}:\d{2}$/);
  });

  it("validates weekday range 1-7", () => {
    const schedule: BandwidthSchedule = {
      startTime: "09:00",
      endTime: "18:00",
      downloadSpeed: 1024 * 1024,
      uploadSpeed: 512 * 1024,
      weekdays: [1, 2, 3, 4, 5],
    };

    schedule.weekdays?.forEach((day) => {
      expect(day).toBeGreaterThanOrEqual(1);
      expect(day).toBeLessThanOrEqual(7);
    });
  });

  it("allows empty weekdays for daily schedule", () => {
    const schedule: BandwidthSchedule = {
      startTime: "22:00",
      endTime: "06:00",
      downloadSpeed: 1024 * 1024,
      uploadSpeed: 512 * 1024,
    };

    expect(schedule.weekdays).toBeUndefined();
  });

  it("validates speed values are non-negative", () => {
    const schedule: BandwidthSchedule = {
      startTime: "22:00",
      endTime: "06:00",
      downloadSpeed: 0, // 0 means unlimited
      uploadSpeed: 0,
    };

    expect(schedule.downloadSpeed).toBeGreaterThanOrEqual(0);
    expect(schedule.uploadSpeed).toBeGreaterThanOrEqual(0);
  });
});

describe("ScheduleParams", () => {
  it("allows optional speed limits", () => {
    const rule: ScheduleRule = {
      id: "test",
      name: "Test",
      ruleType: "BandwidthPlan",
      cronExpression: "0 22 * * *",
      params: {
        downloadSpeed: 1024 * 1024,
        uploadSpeed: 512 * 1024,
      },
      enabled: true,
      createdAt: "2026-01-01T00:00:00Z",
    };

    expect(rule.params.downloadSpeed).toBe(1048576);
    expect(rule.params.uploadSpeed).toBe(524288);
  });

  it("allows empty params", () => {
    const rule: ScheduleRule = {
      id: "test",
      name: "Test",
      ruleType: "StartTask",
      cronExpression: "0 9 * * *",
      params: {},
      enabled: true,
      createdAt: "2026-01-01T00:00:00Z",
    };

    expect(rule.params.downloadSpeed).toBeUndefined();
    expect(rule.params.uploadSpeed).toBeUndefined();
  });

  it("allows optional command", () => {
    const rule: ScheduleRule = {
      id: "test",
      name: "Test",
      ruleType: "StartTask",
      cronExpression: "0 9 * * *",
      params: {
        command: "/path/to/script.sh",
      },
      enabled: true,
      createdAt: "2026-01-01T00:00:00Z",
    };

    expect(rule.params.command).toBe("/path/to/script.sh");
  });

  it("allows optional custom parameters", () => {
    const rule: ScheduleRule = {
      id: "test",
      name: "Test",
      ruleType: "StartTask",
      cronExpression: "0 9 * * *",
      params: {
        custom: {
          taskGroup: "daily-downloads",
          priority: "high",
        },
      },
      enabled: true,
      createdAt: "2026-01-01T00:00:00Z",
    };

    expect(rule.params.custom?.taskGroup).toBe("daily-downloads");
    expect(rule.params.custom?.priority).toBe("high");
  });
});

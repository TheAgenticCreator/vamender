// SPDX-License-Identifier: MIT

using System;
using MVR.FileManagementSecure;

namespace VaMenderPlugin
{
    internal sealed class EngineClient
    {
        private const string StateDirectory =
            "Saves/PluginData/VaMender/Bridge";
        private const string RequestPath =
            StateDirectory + "/request.json";
        private const string StatusPath =
            StateDirectory + "/status.txt";
        private const string HeartbeatPath =
            StateDirectory + "/heartbeat.txt";

        private readonly Action<string, string> _setStatus;
        private readonly Action _rescanPackages;
        private readonly Action<bool> _setBusy;
        private string _pendingOperation;
        private string _lastEngineStatus;
        private string _statusAtQueue;
        private bool _operationInFlight;

        public EngineClient(
            Action<string, string> setStatus,
            Action rescanPackages,
            Action<bool> setBusy)
        {
            _setStatus = setStatus;
            _rescanPackages = rescanPackages;
            _setBusy = setBusy;
        }

        public void RefreshStatus()
        {
            try
            {
                string status = FileManagerSecure.FileExists(
                    StatusPath,
                    false)
                    ? FileManagerSecure.ReadAllText(StatusPath)
                    : "READY: VaMender installed engine is online.";
                if (_operationInFlight &&
                    IsTerminal(status) &&
                    status != _statusAtQueue)
                {
                    _operationInFlight = false;
                }
                SetBusy(
                    IsWorking(status) ||
                    FileManagerSecure.FileExists(RequestPath, false) ||
                    _pendingOperation != null ||
                    _operationInFlight);
                if (!FileManagerSecure.FileExists(HeartbeatPath, false))
                {
                    if (IsWorking(status))
                    {
                        WorkingWithDelayedHeartbeat(status);
                        return;
                    }
                    Offline();
                    return;
                }
                DateTime heartbeat =
                    FileManagerSecure.FileLastWriteTime(
                        HeartbeatPath,
                        false);
                if ((DateTime.Now - heartbeat).TotalSeconds > 5.0)
                {
                    if (IsWorking(status))
                    {
                        WorkingWithDelayedHeartbeat(status);
                        return;
                    }
                    Offline();
                    return;
                }
                _setStatus("ENGINE ONLINE", status);
                if (status != _lastEngineStatus &&
                    status.StartsWith(
                        "COMPLETE:",
                        StringComparison.OrdinalIgnoreCase) &&
                    _rescanPackages != null)
                {
                    _rescanPackages();
                }
                _lastEngineStatus = status;
            }
            catch (Exception exception)
            {
                _setStatus(
                    "ENGINE STATUS ERROR",
                    "Cannot read the VaMender installed engine: " +
                    exception.Message);
            }
        }

        public void QueueCheck()
        {
            Queue("check", false);
        }

        public void QueueDeepCheck()
        {
            Queue("check", true);
        }

        public void QueuePlan()
        {
            Queue("plan", false);
        }

        public void QueueRepair()
        {
            Queue("repair", false);
        }

        public void QueueMigration()
        {
            Queue("migrate", false);
        }

        public void QueueFullRun()
        {
            Queue("run", false);
        }

        public void QueueRestoreLast()
        {
            Queue("restore-last", false);
        }

        public void QueueRestoreAll()
        {
            Queue("restore-all", false);
        }

        private void Queue(string operation, bool deep)
        {
            try
            {
                if (!EngineOnline())
                {
                    Offline();
                    return;
                }
                if (FileManagerSecure.FileExists(RequestPath, false))
                {
                    SetBusy(true);
                    _setStatus(
                        "ENGINE BUSY",
                        "A VaMender request is already queued. Wait for it " +
                        "to finish before starting another operation.");
                    return;
                }
                string current = FileManagerSecure.FileExists(
                    StatusPath,
                    false)
                    ? FileManagerSecure.ReadAllText(StatusPath)
                    : "";
                if (current.StartsWith(
                        "RUNNING:",
                        StringComparison.OrdinalIgnoreCase) ||
                    current.StartsWith(
                        "WAITING:",
                        StringComparison.OrdinalIgnoreCase))
                {
                    SetBusy(true);
                    _setStatus("ENGINE BUSY", current);
                    return;
                }

                _pendingOperation = operation;
                _statusAtQueue = current;
                _operationInFlight = true;
                SetBusy(true);
                string identifier =
                    DateTime.UtcNow.Ticks.ToString();
                string request =
                    "{\n" +
                    "  \"id\": \"" + identifier + "\",\n" +
                    "  \"operation\": \"" + operation + "\",\n" +
                    "  \"deep\": " +
                    (deep ? "true" : "false") + "\n" +
                    "}\n";
                FileManagerSecure.WriteAllText(
                    RequestPath,
                    request,
                    RequestAccepted,
                    RequestDenied,
                    RequestFailed);
                _setStatus(
                    "REQUESTING " + operation.ToUpperInvariant(),
                    "VaMender is sending the operation to its installed engine. " +
                    "VaM remains open. Changed VARs are backed up first, and " +
                    "VaM's package registry will be rescanned afterward.");
            }
            catch (Exception exception)
            {
                RequestFailed(exception);
            }
        }

        private bool EngineOnline()
        {
            string status = FileManagerSecure.FileExists(
                StatusPath,
                false)
                ? FileManagerSecure.ReadAllText(StatusPath)
                : "";
            if (IsWorking(status))
            {
                return true;
            }
            if (!FileManagerSecure.FileExists(HeartbeatPath, false))
            {
                return false;
            }
            DateTime heartbeat =
                FileManagerSecure.FileLastWriteTime(HeartbeatPath, false);
            return (DateTime.Now - heartbeat).TotalSeconds <= 5.0;
        }

        private static bool IsWorking(string status)
        {
            return status.StartsWith(
                    "RUNNING:",
                    StringComparison.OrdinalIgnoreCase) ||
                status.StartsWith(
                    "WAITING:",
                    StringComparison.OrdinalIgnoreCase);
        }

        private static bool IsTerminal(string status)
        {
            return status.StartsWith(
                    "COMPLETE:",
                    StringComparison.OrdinalIgnoreCase) ||
                status.StartsWith(
                    "FAILED:",
                    StringComparison.OrdinalIgnoreCase);
        }

        private void WorkingWithDelayedHeartbeat(string status)
        {
            SetBusy(true);
            _setStatus(
                "ENGINE WORKING",
                status + "\n\nThis operation is still running across the " +
                "AddonPackages library. VaMender will publish its result and " +
                "rescan VaM when it finishes.");
            _lastEngineStatus = status;
        }

        private void RequestAccepted()
        {
            string operation = _pendingOperation;
            _pendingOperation = null;
            SetBusy(true);
            _setStatus(
                "QUEUED " + operation.ToUpperInvariant(),
                "Request accepted. Keep VaM open and watch this status panel. " +
                "VaMender will ask VaM to rescan packages when the operation " +
                "finishes.");
        }

        private void RequestDenied()
        {
            _pendingOperation = null;
            _operationInFlight = false;
            SetBusy(false);
            _setStatus(
                "REQUEST DENIED",
                "VaM denied the secure engine request. No repair was run " +
                "and VaM will remain open.");
        }

        private void RequestFailed(Exception exception)
        {
            _pendingOperation = null;
            _operationInFlight = false;
            SetBusy(false);
            _setStatus(
                "REQUEST FAILED",
                "Cannot queue the VaMender operation: " +
                exception.Message);
        }

        private void SetBusy(bool busy)
        {
            if (_setBusy != null)
            {
                _setBusy(busy);
            }
        }

        private void Offline()
        {
            _pendingOperation = null;
            _operationInFlight = false;
            SetBusy(false);
            _setStatus(
                "ENGINE OFFLINE",
                "VaMender's installed engine is offline. Repair or reinstall " +
                "VaMender from the latest beta Setup build, then press " +
                "Refresh Engine Status. No PowerShell script or open console " +
                "is required.");
        }
    }
}

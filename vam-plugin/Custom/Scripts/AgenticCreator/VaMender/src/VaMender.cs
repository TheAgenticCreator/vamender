// SPDX-License-Identifier: MIT

using System;
using System.Collections.Generic;
using UnityEngine;

namespace VaMenderPlugin
{
    public class VaMender : MVRScript
    {
        private const string Version = "0.2.0";
        private const string ReleaseUrl =
            "https://github.com/TheAgenticCreator/vamender/releases?beta=1";

        private JSONStorableString _status;
        private JSONStorableString _details;
        private EngineClient _engineClient;
        private NativePackageService _nativePackages;
        private DefaultSceneLauncher _defaultSceneLauncher;
        private PluginUiNavigator _uiNavigator;
        private readonly List<MomentaryAction> _operationActions =
            new List<MomentaryAction>();
        private float _nextDefaultLauncherCheck;
        private float _nextEngineCheck;
        private string _lastStatus;
        private string _lastStatusDetails;
        private bool _operationBusy;

        public override void Init()
        {
            try
            {
                BuildUi();
                _nativePackages = new NativePackageService(SetStatus);
                _engineClient = new EngineClient(
                    SetStatus,
                    RescanPackages,
                    SetOperationBusy);
                _uiNavigator = new PluginUiNavigator(this, SetStatus);
                _defaultSceneLauncher =
                    new DefaultSceneLauncher(_uiNavigator.RequestOpen);
                _defaultSceneLauncher.EnsureInstalled();
                _engineClient.RefreshStatus();
            }
            catch (Exception exception)
            {
                SuperController.LogError(
                    "VaMender native plugin initialization failed: " +
                    exception);
            }
        }

        private void BuildUi()
        {
            _status = new JSONStorableString(
                "Status",
                "VaMender " + Version + " — initializing");
            RegisterString(_status);
            UIDynamicTextField statusField = CreateTextField(_status, false);
            statusField.height = 100f;

            _details = new JSONStorableString(
                "Engine activity",
                "Connecting to the installed VaMender engine...");
            RegisterString(_details);
            UIDynamicTextField detailsField = CreateTextField(_details, true);
            detailsField.height = 420f;

            JSONStorableString integration = new JSONStorableString(
                "VaMender operations",
                "FULL VAMENDER CONTROL PANEL\n\n" +
                "These buttons run the complete VaMender engine installed by VaMender Setup. " +
                "VaM stays open for Check, Plan, Repair, Migrate, Full " +
                "Optimize, and Restore. Every changed or archived VAR is " +
                "SHA-256 backed up first. When an operation completes, " +
                "VaMender asks VaM to rescan its package registry.");
            RegisterString(integration);
            UIDynamicTextField integrationField =
                CreateTextField(integration, false);
            integrationField.height = 260f;

            AddOperationAction(
                "VaMender: Check Library",
                false,
                QueueCheck);
            AddOperationAction(
                "VaMender: Deep CRC Check",
                false,
                QueueDeepCheck);
            AddOperationAction(
                "VaMender: Build Cleanup Plan",
                false,
                QueuePlan);
            AddOperationAction(
                "VaMender: Repair VARs",
                false,
                QueueRepair);
            AddOperationAction(
                "VaMender: Clean Old Versions",
                false,
                QueueMigration);
            AddOperationAction(
                "VaMender: Full Optimize",
                false,
                QueueFullRun);
            AddOperationAction(
                "VaMender: Restore Most Recent VAR",
                false,
                QueueRestoreLast);
            AddOperationAction(
                "VaMender: Restore All Backups",
                false,
                QueueRestoreAll);
            AddAction(
                "VaMender: Refresh Engine Status",
                false,
                RefreshEngineStatus);

            JSONStorableString setup = new JSONStorableString(
                "VaMender engine",
                "VaMender Setup installs the backup-first engine and this plugin " +
                "together. No PowerShell script or open console is required. " +
                "The engine runs as a private per-user background component " +
                "because VaM blocks archive replacement inside ordinary script " +
                "plugins. VaM remains open and rescans after engine changes.\n\n" +
                "Release: " + ReleaseUrl);
            RegisterString(setup);
            UIDynamicTextField setupField =
                CreateTextField(setup, true);
            setupField.height = 340f;

            JSONStorableString helpers = new JSONStorableString(
                "Optional VaM helpers",
                "These two convenience actions are performed by VaM itself. " +
                "They are not VaMender repair operations.");
            RegisterString(helpers);
            UIDynamicTextField helpersField =
                CreateTextField(helpers, true);
            helpersField.height = 130f;
            AddAction(
                "Helper: Open VaM Package Manager",
                true,
                OpenPackageManager);
            AddAction(
                "Helper: Rescan Packages in VaM",
                true,
                RescanPackages);
        }

        private MomentaryAction AddAction(
            string name,
            bool rightSide,
            JSONStorableAction.ActionCallback callback)
        {
            MomentaryAction action =
                new MomentaryAction(name, callback);
            RegisterAction(action.Action);
            UIDynamicButton button = CreateButton(name, rightSide);
            action.Bind(button, rightSide);
            return action;
        }

        private void AddOperationAction(
            string name,
            bool rightSide,
            JSONStorableAction.ActionCallback callback)
        {
            _operationActions.Add(
                AddAction(name, rightSide, callback));
        }

        private void SetOperationBusy(bool busy)
        {
            if (_operationBusy == busy)
            {
                return;
            }
            _operationBusy = busy;
            foreach (MomentaryAction action in _operationActions)
            {
                action.SetInteractable(!busy);
            }
        }

        private void OpenPackageManager()
        {
            try
            {
                SuperController.singleton.OpenPackageManager();
                SetStatus(
                    "READY",
                    "Opened VaM's built-in Add-On Package Manager.");
            }
            catch (Exception exception)
            {
                SetStatus(
                    "ERROR",
                    "Cannot open VaM's Package Manager: " +
                    exception.Message);
            }
        }

        private void RescanPackages()
        {
            _nativePackages.RescanPackages();
        }

        private void QueueCheck()
        {
            _engineClient.QueueCheck();
        }

        private void QueueDeepCheck()
        {
            _engineClient.QueueDeepCheck();
        }

        private void QueuePlan()
        {
            _engineClient.QueuePlan();
        }

        private void QueueRepair()
        {
            _engineClient.QueueRepair();
        }

        private void QueueMigration()
        {
            _engineClient.QueueMigration();
        }

        private void QueueFullRun()
        {
            _engineClient.QueueFullRun();
        }

        private void QueueRestoreLast()
        {
            _engineClient.QueueRestoreLast();
        }

        private void QueueRestoreAll()
        {
            _engineClient.QueueRestoreAll();
        }

        private void RefreshEngineStatus()
        {
            _engineClient.RefreshStatus();
        }

        private void SetStatus(string status, string details)
        {
            if (_lastStatus == status && _lastStatusDetails == details)
            {
                return;
            }
            _lastStatus = status;
            _lastStatusDetails = details;
            _status.val = "VAMENDER — " + status;
            _details.val = details;
            SuperController.LogMessage(
                "VaMender: " + status + " — " + details);
        }

        private void Update()
        {
            if (_defaultSceneLauncher != null &&
                Time.unscaledTime >= _nextDefaultLauncherCheck)
            {
                _nextDefaultLauncherCheck = Time.unscaledTime + 3f;
                _defaultSceneLauncher.EnsureInstalled();
            }
            if (_engineClient != null &&
                Time.unscaledTime >= _nextEngineCheck)
            {
                _nextEngineCheck = Time.unscaledTime + 2f;
                _engineClient.RefreshStatus();
            }
            if (_uiNavigator != null)
            {
                _uiNavigator.Update();
            }
        }

        private void OnDestroy()
        {
            if (_defaultSceneLauncher != null)
            {
                _defaultSceneLauncher.Dispose();
                _defaultSceneLauncher = null;
            }
        }
    }
}

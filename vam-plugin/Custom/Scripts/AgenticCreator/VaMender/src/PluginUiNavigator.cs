// SPDX-License-Identifier: MIT

using System;
using UnityEngine;

namespace VaMenderPlugin
{
    internal sealed class PluginUiNavigator
    {
        private const string MainMenuContentPath =
            "WorldScaleAdjust/HUD/MainUICanvas/Panel/Content";
        private const string SessionPluginsTab = "TabSessionPlugins";

        private readonly MVRScript _owner;
        private readonly Action<string, string> _setStatus;
        private int _stage;
        private float _nextAttempt;
        private float _deadline;

        public PluginUiNavigator(
            MVRScript owner,
            Action<string, string> setStatus)
        {
            _owner = owner;
            _setStatus = setStatus;
        }

        public void RequestOpen()
        {
            SuperController controller = SuperController.singleton;
            if (controller == null)
            {
                _setStatus(
                    "OPEN FAILED",
                    "VaM's SuperController is unavailable.");
                return;
            }
            if (_owner.UITransform == null)
            {
                _setStatus(
                    "OPEN FAILED",
                    "VaMender's custom UI has not been created yet.");
                return;
            }

            controller.ShowMainHUDAuto();
            controller.activeUI = SuperController.ActiveUI.MainMenu;
            _stage = 1;
            _nextAttempt = Time.unscaledTime + 0.15f;
            _deadline = Time.unscaledTime + 3f;
        }

        public void Update()
        {
            if (_stage == 0 || Time.unscaledTime < _nextAttempt)
            {
                return;
            }

            try
            {
                if (_stage == 1)
                {
                    if (TrySelectSessionPlugins())
                    {
                        _stage = 2;
                        _nextAttempt = Time.unscaledTime + 0.1f;
                    }
                    else
                    {
                        RetryOrFail();
                    }
                }
                else if (_stage == 2)
                {
                    ActivateOwnerUi();
                    _stage = 3;
                    _nextAttempt = Time.unscaledTime + 0.1f;
                }
                else if (_owner.UITransform.gameObject.activeInHierarchy)
                {
                    _stage = 0;
                    _setStatus(
                        "READY",
                        "VaMender is open. This is the VaMender control " +
                        "panel, not VaM's File menu.");
                }
                else
                {
                    TrySelectSessionPlugins();
                    ActivateOwnerUi();
                    RetryOrFail();
                }
            }
            catch (Exception exception)
            {
                _stage = 0;
                _setStatus(
                    "OPEN FAILED",
                    "Cannot activate VaMender's panel: " +
                    exception.Message);
            }
        }

        private void RetryOrFail()
        {
            if (Time.unscaledTime < _deadline)
            {
                _nextAttempt = Time.unscaledTime + 0.1f;
                return;
            }
            _stage = 0;
            _setStatus(
                "OPEN FAILED",
                "VaM did not expose and activate its Session Plugins panel " +
                "within three seconds. Open Session Plugins once, then use " +
                "Open VaMender again.");
        }

        private bool TrySelectSessionPlugins()
        {
            SuperController controller = SuperController.singleton;
            UITabSelector selector = FindMainMenuSelector(controller);
            if (selector == null)
            {
                return false;
            }
            selector.SetActiveTab(SessionPluginsTab);
            return true;
        }

        private void ActivateOwnerUi()
        {
            if (_owner.manager != null &&
                _owner.manager.pluginContainer != null)
            {
                MVRScript[] scripts =
                    _owner.manager.pluginContainer
                        .GetComponentsInChildren<MVRScript>(true);
                for (int index = 0; index < scripts.Length; index++)
                {
                    MVRScript script = scripts[index];
                    if (script != null &&
                        script != _owner &&
                        script.UITransform != null)
                    {
                        script.UITransform.gameObject.SetActive(false);
                    }
                }
            }

            _owner.UITransform.gameObject.SetActive(true);
        }

        private static UITabSelector FindMainMenuSelector(
            SuperController controller)
        {
            GameObject content = GameObject.Find(MainMenuContentPath);
            if (content != null)
            {
                UITabSelector selector =
                    content.GetComponent<UITabSelector>();
                if (selector != null)
                {
                    return selector;
                }
            }
            return controller.mainMenuTabSelector;
        }
    }
}

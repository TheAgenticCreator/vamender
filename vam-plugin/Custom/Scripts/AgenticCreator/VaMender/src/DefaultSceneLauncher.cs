// SPDX-License-Identifier: MIT

using System;
using UnityEngine;
using UnityEngine.UI;

namespace VaMenderPlugin
{
    internal sealed class DefaultSceneLauncher : IDisposable
    {
        private const string LauncherName =
            "AgenticCreator.VaMender.DefaultSceneLauncher";
        private const string AnchorLabel = "Open Default Scene";

        private readonly Action _openVaMender;
        private GameObject _launcher;

        public DefaultSceneLauncher(Action openVaMender)
        {
            _openVaMender = openVaMender;
        }

        public bool EnsureInstalled()
        {
            if (_launcher != null)
            {
                return true;
            }

            Button[] buttons = UnityEngine.Object.FindObjectsOfType<Button>();
            Button anchor = null;
            for (int index = 0; index < buttons.Length; index++)
            {
                Button button = buttons[index];
                if (button == null)
                {
                    continue;
                }
                if (button.gameObject.name == LauncherName)
                {
                    _launcher = button.gameObject;
                    return true;
                }
                Text label = button.GetComponentInChildren<Text>(true);
                if (label != null &&
                    string.Equals(
                        Normalize(label.text),
                        Normalize(AnchorLabel),
                        StringComparison.OrdinalIgnoreCase))
                {
                    anchor = button;
                }
            }
            if (anchor == null || anchor.transform.parent == null)
            {
                return false;
            }

            Button launcher = UnityEngine.Object.Instantiate(
                anchor,
                anchor.transform.parent,
                false);
            launcher.gameObject.name = LauncherName;
            launcher.onClick = new Button.ButtonClickedEvent();
            launcher.onClick.AddListener(OpenVaMender);
            launcher.transform.SetSiblingIndex(
                anchor.transform.GetSiblingIndex());

            Text[] labels = launcher.GetComponentsInChildren<Text>(true);
            for (int index = 0; index < labels.Length; index++)
            {
                labels[index].text = "Open\nVaMender";
                labels[index].resizeTextForBestFit = true;
            }

            PlaceLeftOfButtonRow(anchor, launcher);
            _launcher = launcher.gameObject;
            return true;
        }

        private static void PlaceLeftOfButtonRow(
            Button anchor,
            Button launcher)
        {
            RectTransform anchorRect = anchor.transform as RectTransform;
            RectTransform launcherRect = launcher.transform as RectTransform;
            Transform parent = anchor.transform.parent;
            if (anchorRect == null || launcherRect == null || parent == null)
            {
                return;
            }

            float anchorWidth = Math.Max(anchorRect.rect.width, 1f);
            float anchorHeight = Math.Max(anchorRect.rect.height, 1f);
            float minimumLeft =
                anchorRect.anchoredPosition.x - (anchorWidth * 0.5f);
            for (int index = 0; index < parent.childCount; index++)
            {
                RectTransform sibling = parent.GetChild(index) as RectTransform;
                if (sibling == null || sibling == launcherRect)
                {
                    continue;
                }
                if (Math.Abs(
                        sibling.anchoredPosition.y -
                        anchorRect.anchoredPosition.y) >
                    anchorHeight)
                {
                    continue;
                }
                float siblingWidth = Math.Max(sibling.rect.width, 1f);
                minimumLeft = Math.Min(
                    minimumLeft,
                    sibling.anchoredPosition.x - (siblingWidth * 0.5f));
            }

            launcherRect.anchoredPosition = new Vector2(
                minimumLeft - 12f - (anchorWidth * 0.5f),
                anchorRect.anchoredPosition.y);
        }

        private static string Normalize(string value)
        {
            if (value == null)
            {
                return "";
            }
            return value
                .Replace("\r", " ")
                .Replace("\n", " ")
                .Trim();
        }

        private void OpenVaMender()
        {
            if (_openVaMender != null)
            {
                _openVaMender();
            }
        }

        public void Dispose()
        {
            if (_launcher != null)
            {
                UnityEngine.Object.Destroy(_launcher);
                _launcher = null;
            }
        }
    }
}

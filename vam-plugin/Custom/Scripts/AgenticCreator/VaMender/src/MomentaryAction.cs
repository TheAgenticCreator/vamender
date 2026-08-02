// SPDX-License-Identifier: MIT

namespace VaMenderPlugin
{
    internal sealed class MomentaryAction
    {
        private readonly JSONStorableAction.ActionCallback _callback;
        private UIDynamicButton _button;

        public MomentaryAction(
            string name,
            JSONStorableAction.ActionCallback callback)
        {
            _callback = callback;
            Action = new JSONStorableAction(name, Invoke);
        }

        public JSONStorableAction Action { get; private set; }

        public void Bind(UIDynamicButton button, bool rightSide)
        {
            _button = button;
            Action.RegisterButton(button, rightSide);
        }

        public void SetInteractable(bool interactable)
        {
            Action.interactable = interactable;
            ReleaseVisual();
        }

        private void Invoke()
        {
            try
            {
                if (_callback != null)
                {
                    _callback();
                }
            }
            finally
            {
                ReleaseVisual();
            }
        }

        private void ReleaseVisual()
        {
            if (_button != null && _button.button != null)
            {
                _button.button.OnDeselect(null);
            }
        }
    }
}
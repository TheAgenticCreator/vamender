// SPDX-License-Identifier: MIT

using System;

namespace VaMenderPlugin
{
    internal sealed class NativePackageService
    {
        private readonly Action<string, string> _setStatus;

        public NativePackageService(Action<string, string> setStatus)
        {
            _setStatus = setStatus;
        }

        public void RescanPackages()
        {
            try
            {
                Controller().RescanPackages();
                _setStatus(
                    "COMPLETE",
                    "VaM completed its native AddonPackages rescan.");
            }
            catch (Exception exception)
            {
                Fail("rescan packages", exception);
            }
        }

        private static SuperController Controller()
        {
            SuperController controller = SuperController.singleton;
            if (controller == null)
            {
                throw new InvalidOperationException(
                    "VaM's SuperController is unavailable.");
            }
            return controller;
        }

        private void Fail(string operation, Exception exception)
        {
            _setStatus(
                "ERROR",
                "Cannot " + operation + ": " + exception.Message);
        }
    }
}

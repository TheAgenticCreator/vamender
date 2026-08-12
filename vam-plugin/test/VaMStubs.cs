// SPDX-License-Identifier: MIT

using System;
using System.Collections.Generic;

namespace UnityEngine
{
    public class Object
    {
        public static T[] FindObjectsOfType<T>()
        {
            return new T[0];
        }

        public static T Instantiate<T>(
            T original,
            Transform parent,
            bool worldPositionStays)
            where T : Object, new()
        {
            return new T();
        }

        public static void Destroy(Object value)
        {
        }
    }

    public class Component : Object
    {
        public GameObject gameObject;
        public Transform transform;

        public T GetComponentInChildren<T>(bool includeInactive)
            where T : new()
        {
            return new T();
        }

        public T GetComponent<T>()
            where T : new()
        {
            return new T();
        }

        public T[] GetComponentsInChildren<T>(bool includeInactive)
        {
            return new T[0];
        }
    }

    public class GameObject : Object
    {
        public string name = "";
        public Transform transform;
        public bool activeInHierarchy;

        public static GameObject Find(string name)
        {
            return new GameObject();
        }

        public T GetComponent<T>()
            where T : new()
        {
            return new T();
        }

        public void SetActive(bool active)
        {
            activeInHierarchy = active;
        }
    }

    public class Transform : Component
    {
        public Transform parent;
        public int childCount;

        public void SetAsLastSibling()
        {
        }

        public int GetSiblingIndex()
        {
            return 0;
        }

        public void SetSiblingIndex(int index)
        {
        }

        public Transform GetChild(int index)
        {
            return new Transform();
        }
    }

    public struct Vector2
    {
        public float x;
        public float y;

        public Vector2(float xValue, float yValue)
        {
            x = xValue;
            y = yValue;
        }
    }

    public class RectTransform : Transform
    {
        public Rect rect;
        public Vector2 anchoredPosition;
    }

    public static class Application
    {
        public static string dataPath = "";
        public static string persistentDataPath = "";

        public static void OpenURL(string url)
        {
        }
    }

    public static class Time
    {
        public static float unscaledTime;
    }

    public struct Rect
    {
        public float width;
        public float height;

        public Rect(float x, float y, float width, float height)
        {
            this.width = width;
            this.height = height;
        }
    }

}

namespace UnityEngine.Events
{
    public delegate void UnityAction();
}

namespace UnityEngine.UI
{
    public class Button : UnityEngine.Component
    {
        public ButtonClickedEvent onClick = new ButtonClickedEvent();
        public int deselectCount;

        public void OnDeselect(object eventData)
        {
            deselectCount++;
        }

        public class ButtonClickedEvent
        {
            private UnityEngine.Events.UnityAction _listeners;

            public void AddListener(UnityEngine.Events.UnityAction action)
            {
                _listeners += action;
            }

            public void RemoveAllListeners()
            {
                _listeners = null;
            }

            public void Invoke()
            {
                if (_listeners != null)
                {
                    _listeners();
                }
            }
        }
    }

    public class Text : UnityEngine.Component
    {
        public string text = "";
        public bool resizeTextForBestFit;
    }
}

namespace SimpleJSON
{
    public class JSONNode
    {
        public virtual string Value { get; set; }

        public virtual JSONNode this[string key]
        {
            get { return new JSONNode(); }
        }
    }

    public class JSONClass : JSONNode
    {
    }
}

public class JSONStorable
{
    public UnityEngine.Transform UITransform = new UnityEngine.Transform();

    public void RegisterString(JSONStorableString value)
    {
    }

    public void RegisterBool(JSONStorableBool value)
    {
    }

    public void RegisterAction(JSONStorableAction value)
    {
    }
}

public class MVRScript : JSONStorable
{
    public string name = "";
    public MVRPluginManager manager = new MVRPluginManager();

    public virtual void Init()
    {
    }

    public UIDynamicTextField CreateTextField(
        JSONStorableString value,
        bool rightSide)
    {
        return new UIDynamicTextField();
    }

    public UIDynamicToggle CreateToggle(
        JSONStorableBool value,
        bool rightSide)
    {
        return new UIDynamicToggle();
    }

    public UIDynamicButton CreateButton(string label, bool rightSide)
    {
        return new UIDynamicButton();
    }
}

public class JSONStorableString
{
    public JSONStorableString(string name, string startingValue)
    {
        val = startingValue;
    }

    public string val;
}

public class JSONStorableBool
{
    public JSONStorableBool(string name, bool startingValue)
    {
        val = startingValue;
    }

    public bool val;
}

public class JSONStorableAction
{
    public delegate void ActionCallback();

    private readonly ActionCallback _callback;
    public bool interactable = true;

    public JSONStorableAction(string name, ActionCallback callback)
    {
        _callback = callback;
    }

    public void RegisterButton(UIDynamicButton button, bool rightSide)
    {
        button.button.onClick.AddListener(Invoke);
    }

    private void Invoke()
    {
        if (_callback != null)
        {
            _callback();
        }
    }
}

public class UIDynamic
{
    public float height;
}

public class UIDynamicTextField : UIDynamic
{
}

public class UIDynamicToggle : UIDynamic
{
}

public class UIDynamicButton : UIDynamic
{
    public UnityEngine.UI.Button button = new UnityEngine.UI.Button();
}

public class Atom
{
    public string uid = "";
    public string type = "";
}

public class SuperController
{
    public enum ActiveUI
    {
        None,
        MainMenu,
        PackageManager,
        PackageBuilder,
        PackageDownloader,
        OnlineBrowser
    }

    public static SuperController singleton = new SuperController();
    public static int LogMessageCount;
    public bool isLoading;
    public string LoadedSceneName = "";
    public bool disablePackages;
    public bool disableLoadSceneButton;
    public bool disableSaveSceneButton;
    public UITabSelector mainMenuTabSelector = new UITabSelector();
    public UnityEngine.Transform packageManagerUI =
        new UnityEngine.Transform();
    public MVR.Hub.HubDownloader packageDownloader =
        new MVR.Hub.HubDownloader();
    public MVR.Hub.HubBrowse hubBrowser =
        new MVR.Hub.HubBrowse();
    public bool hubDisabled;
    public ActiveUI activeUI;

    public static void LogError(string value)
    {
    }

    public static void LogMessage(string value)
    {
        LogMessageCount++;
    }

    public List<Atom> GetAtoms()
    {
        return new List<Atom>();
    }

    public void UnloadUnusedResources()
    {
    }

    public void OpenPackageManager()
    {
    }

    public void OpenPackageDownloader()
    {
    }

    public void OpenHub()
    {
    }

    public void RescanPackages()
    {
    }

    public void SetMainMenuTab(string tabName)
    {
    }

    public void ShowMainHUDAuto()
    {
    }

    public void SetActiveUI(string uiName)
    {
    }

    public void Quit()
    {
    }
}

namespace MVR.Hub
{
    public class HubBrowse
    {
        public void OpenMissingPackagesPanel()
        {
        }

        public void OpenUpdatesPanel()
        {
        }

        public void DownloadAllUpdates()
        {
        }
    }

    public class HubDownloader
    {
        public delegate void ErrorCallback(string error);
        public delegate void SuccessCallback();

        public bool DownloadAllMissingPackages(
            SuccessCallback successCallback,
            ErrorCallback errorCallback)
        {
            return true;
        }
    }
}

public class UITabSelector
{
    public bool HasTab(string tabName)
    {
        return true;
    }

    public void SetActiveTab(string tabName)
    {
    }
}

public class MVRPluginManager : UnityEngine.Object
{
    protected List<MVRPlugin> plugins = new List<MVRPlugin>();
    public UnityEngine.Transform pluginListPanel =
        new UnityEngine.Transform();
    public UnityEngine.Transform pluginContainer =
        new UnityEngine.Transform();

    public SimpleJSON.JSONClass GetJSON()
    {
        return new SimpleJSON.JSONClass();
    }
}

public class MVRScriptControllerUI
{
    public UnityEngine.UI.Text label = new UnityEngine.UI.Text();
    public UnityEngine.UI.Button openUIButton =
        new UnityEngine.UI.Button();
}

public class MVRPlugin
{
    public JSONStorableUrl pluginURLJSON = new JSONStorableUrl();
}

public class JSONStorableUrl
{
    public string val = "";
}

public class DAZDynamicItem : UnityEngine.Object
{
    public bool active;
    public string packageUid = "";
}

public class DAZMorphBank : UnityEngine.Object
{
    protected HashSet<string> currentLoadedMorphPackageUids =
        new HashSet<string>();
}

namespace MVR.FileManagement
{
    public class VarPackage
    {
        public string Uid = "";
        public bool IsUnpacking;
        public bool IsRepacking;
    }

    public static class FileManager
    {
        public static Dictionary<string, byte[]> TestBytes =
            new Dictionary<string, byte[]>();
        public static Dictionary<string, string> TestText =
            new Dictionary<string, string>();
        public static string CurrentPackageUid = "";
        public static string TopPackageUid = "";

        public static List<VarPackage> GetPackages()
        {
            return new List<VarPackage>();
        }

        public static void UnregisterPackage(VarPackage package)
        {
        }

        public static void Refresh()
        {
        }

        public static byte[] ReadAllBytes(string path, bool restrictPath)
        {
            return TestBytes[path];
        }

        public static string ReadAllText(string path, bool restrictPath)
        {
            return TestText[path];
        }
    }
}

namespace MVR.FileManagementSecure
{
    public delegate void UserActionCallback();
    public delegate void ExceptionCallback(System.Exception exception);

    public static class FileManagerSecure
    {
        public static string TestRoot = "";

        public static string GetFullPath(string path)
        {
            return System.IO.Path.IsPathRooted(path)
                ? path
                : System.IO.Path.GetFullPath(
                    System.IO.Path.Combine(TestRoot, path));
        }

        public static bool FileExists(
            string path,
            bool onlySystemFiles = false)
        {
            return System.IO.File.Exists(GetFullPath(path)) ||
                MVR.FileManagement.FileManager.TestBytes.ContainsKey(path) ||
                MVR.FileManagement.FileManager.TestText.ContainsKey(path);
        }

        public static bool DirectoryExists(
            string path,
            bool onlySystemDirectories = false)
        {
            return System.IO.Directory.Exists(GetFullPath(path));
        }

        public static void CreateDirectory(string path)
        {
            System.IO.Directory.CreateDirectory(GetFullPath(path));
        }

        public static byte[] ReadAllBytes(string path)
        {
            if (MVR.FileManagement.FileManager.TestBytes.ContainsKey(path))
            {
                return MVR.FileManagement.FileManager.TestBytes[path];
            }
            return System.IO.File.ReadAllBytes(GetFullPath(path));
        }

        public static string ReadAllText(string path)
        {
            if (MVR.FileManagement.FileManager.TestText.ContainsKey(path))
            {
                return MVR.FileManagement.FileManager.TestText[path];
            }
            return System.IO.File.ReadAllText(GetFullPath(path));
        }

        public static void WriteAllBytes(string path, byte[] bytes)
        {
            System.IO.File.WriteAllBytes(GetFullPath(path), bytes);
        }

        public static void WriteAllText(string path, string text)
        {
            System.IO.File.WriteAllText(GetFullPath(path), text);
        }

        public static void WriteAllText(
            string path,
            string text,
            UserActionCallback successCallback,
            UserActionCallback deniedCallback,
            ExceptionCallback exceptionCallback)
        {
            try
            {
                WriteAllText(path, text);
                if (successCallback != null)
                {
                    successCallback();
                }
            }
            catch (System.Exception exception)
            {
                if (exceptionCallback != null)
                {
                    exceptionCallback(exception);
                }
            }
        }

        public static System.DateTime FileLastWriteTime(
            string path,
            bool onlySystemFiles = false)
        {
            return System.IO.File.GetLastWriteTime(GetFullPath(path));
        }

        public static void DeleteFile(string path)
        {
            System.IO.File.Delete(GetFullPath(path));
        }

        public static void MoveFile(
            string oldPath,
            string newPath,
            bool overwrite = true)
        {
            string destination = GetFullPath(newPath);
            if (overwrite && System.IO.File.Exists(destination))
            {
                System.IO.File.Delete(destination);
            }
            System.IO.File.Move(GetFullPath(oldPath), destination);
        }
    }
}

namespace MeshVR
{
    public class AssetLoader
    {
        protected static AssetLoader singleton = new AssetLoader();
        protected Dictionary<string, object> pathToAssetBundle =
            new Dictionary<string, object>();
    }
}

namespace AssetBundles
{
    public static class AssetBundleManager
    {
        private static Dictionary<string, object> m_LoadedAssetBundles =
            new Dictionary<string, object>();
    }
}

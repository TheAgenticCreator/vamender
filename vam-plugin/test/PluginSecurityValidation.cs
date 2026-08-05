// SPDX-License-Identifier: MIT

using System;
using System.IO;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

internal static class PluginSecurityValidation
{
    private static int _momentaryActionCalls;

    public static int Main(string[] arguments)
    {
        if (arguments.Length != 1)
        {
            throw new InvalidOperationException(
                "Pass the compiled VaMender.dll path.");
        }
        ValidatePluginAssembly(arguments[0]);
        ValidateMomentaryAction();
        return 0;
    }

    private static void ValidateMomentaryAction()
    {
        _momentaryActionCalls = 0;
        VaMenderPlugin.MomentaryAction action =
            new VaMenderPlugin.MomentaryAction(
                "test",
                CountMomentaryAction);
        UIDynamicButton button = new UIDynamicButton();
        action.Bind(button, false);
        button.button.onClick.Invoke();
        Require(
            _momentaryActionCalls == 1,
            "Momentary action did not invoke exactly once.");
        Require(
            button.button.deselectCount == 1,
            "Momentary action did not release its pressed visual.");
        action.SetInteractable(false);
        Require(
            !action.Action.interactable,
            "Momentary action did not lock its VaM control.");
        Require(
            button.button.deselectCount == 2,
            "Locking a momentary action did not clear its pressed visual.");
        Console.WriteLine(
            "Momentary action release and operation-lock validation passed.");
    }

    private static void CountMomentaryAction()
    {
        _momentaryActionCalls++;
    }

    private static void ValidatePluginAssembly(string path)
    {
        using (FileStream stream = File.OpenRead(path))
        using (PEReader portableExecutable = new PEReader(stream))
        {
            MetadataReader metadata = portableExecutable.GetMetadataReader();
            bool referencesVaM = false;
            bool referencesClr2 = false;
            foreach (TypeReferenceHandle handle in metadata.TypeReferences)
            {
                TypeReference reference = metadata.GetTypeReference(handle);
                string typeNamespace =
                    metadata.GetString(reference.Namespace);
                string typeName = metadata.GetString(reference.Name);
                Require(
                    !IsRestrictedNamespace(typeNamespace),
                    "Plugin assembly references prohibited namespace " +
                        typeNamespace + ".");
                Require(
                    !IsRestrictedType(typeNamespace, typeName),
                    "Plugin assembly references prohibited type " +
                        FullTypeName(typeNamespace, typeName) + ".");
            }
            foreach (MemberReferenceHandle handle in
                metadata.MemberReferences)
            {
                MemberReference reference =
                    metadata.GetMemberReference(handle);
                if (reference.Parent.Kind != HandleKind.TypeReference)
                {
                    continue;
                }
                TypeReference parent = metadata.GetTypeReference(
                    (TypeReferenceHandle)reference.Parent);
                string typeNamespace =
                    metadata.GetString(parent.Namespace);
                string typeName = metadata.GetString(parent.Name);
                string memberName = metadata.GetString(reference.Name);
                Require(
                    !IsRestrictedMember(
                        typeNamespace,
                        typeName,
                        memberName),
                    "Plugin assembly references prohibited member " +
                        FullTypeName(typeNamespace, typeName) + "." +
                        memberName + ".");
            }
            int moduleReferenceCount =
                metadata.GetTableRowCount(TableIndex.ModuleRef);
            if (moduleReferenceCount > 0)
            {
                ModuleReferenceHandle handle =
                    MetadataTokens.ModuleReferenceHandle(1);
                ModuleReference reference =
                    metadata.GetModuleReference(handle);
                throw new InvalidOperationException(
                    "Plugin assembly contains unmanaged module reference " +
                    metadata.GetString(reference.Name) + ".");
            }
            foreach (AssemblyReferenceHandle handle in
                metadata.AssemblyReferences)
            {
                AssemblyReference reference =
                    metadata.GetAssemblyReference(handle);
                string name = metadata.GetString(reference.Name);
                if (name == "Assembly-CSharp")
                {
                    referencesVaM = true;
                }
                if (name == "mscorlib" &&
                    reference.Version == new Version(2, 0, 0, 0))
                {
                    referencesClr2 = true;
                }
                Require(
                    !IsRestrictedAssembly(name),
                    "Plugin assembly references prohibited assembly " +
                        name + ".");
            }
            Require(
                referencesVaM,
                "Plugin assembly does not reference VaM's Assembly-CSharp.");
            Require(
                referencesClr2,
                "Plugin assembly must target VaM's CLR 2 profile " +
                    "(mscorlib 2.0.0.0).");
        }
        Console.WriteLine(
            "VaM 1.22.0.13 plugin security metadata validation passed.");
    }

    private static bool IsRestrictedNamespace(string value)
    {
        return IsNamespace(value, "System.IO") ||
            IsNamespace(value, "System.Reflection") ||
            IsNamespace(value, "System.Runtime.InteropServices") ||
            IsNamespace(value, "MVR.FileManagement") ||
            IsNamespace(value, "System.Net") ||
            IsNamespace(value, "UnityEngine.Network") ||
            IsNamespace(value, "UnityEngine.Networking");
    }

    private static bool IsNamespace(string value, string restricted)
    {
        return value == restricted ||
            value.StartsWith(
                restricted + ".",
                StringComparison.Ordinal);
    }

    private static bool IsRestrictedType(
        string typeNamespace,
        string typeName)
    {
        string fullName = FullTypeName(typeNamespace, typeName);
        switch (fullName)
        {
            case "System.AppContext":
            case "System.AppDomain":
            case "System.AppDomainManager":
            case "System.Environment":
            case "System.Diagnostics.Process":
            case "UnityEngine.PlayerPrefs":
            case "UnityEngine.Networking.DownloadHandlerFile":
            case "UnityEngine.Networking.UploadHandlerFile":
            case "System.Xml.XmlReader":
            case "System.Xml.XmlWriter":
            case "System.Xml.XmlTextReader":
            case "System.Xml.XmlTextWriter":
            case "System.Xml.XPath.XPathDocument":
            case "System.Net.WebRequest":
            case "System.Net.WebClient":
                return true;
            default:
                return false;
        }
    }

    private static bool IsRestrictedMember(
        string typeNamespace,
        string typeName,
        string memberName)
    {
        string fullName = FullTypeName(typeNamespace, typeName);
        return (fullName == "UnityEngine.Application" &&
                memberName == "OpenURL") ||
            (fullName == "System.Xml.XmlDocument" &&
             (memberName == "Load" || memberName == "Save")) ||
            (fullName == "System.Delegate" &&
             memberName == "CreateDelegate");
    }

    private static bool IsRestrictedAssembly(string name)
    {
        switch (name)
        {
            case "UnityEditor":
            case "Mono.Cecil":
            case "System.Configuration":
            case "System.Data":
            case "System.Deployment":
            case "System.Dynamic":
            case "System.Management":
            case "System.Resources":
            case "System.Security":
            case "System.Runtime":
            case "System.EnterpriseServices":
                return true;
            default:
                return false;
        }
    }

    private static string FullTypeName(
        string typeNamespace,
        string typeName)
    {
        return string.IsNullOrEmpty(typeNamespace)
            ? typeName
            : typeNamespace + "." + typeName;
    }

    private static void Require(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }
}

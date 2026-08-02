// SPDX-License-Identifier: MIT

using System;
using System.IO;
using System.Reflection;

internal static class VaMTypeLoadValidation
{
    private static string _managedDirectory;

    public static int Main(string[] arguments)
    {
        if (arguments.Length != 2)
        {
            Console.Error.WriteLine(
                "Usage: VaMTypeLoadValidation.exe <VaM Managed> <VaMender.dll>");
            return 2;
        }

        _managedDirectory = Path.GetFullPath(arguments[0]);
        string pluginPath = Path.GetFullPath(arguments[1]);
        AppDomain.CurrentDomain.AssemblyResolve += ResolveAssembly;
        try
        {
            Assembly vam = Assembly.LoadFrom(
                Path.Combine(_managedDirectory, "Assembly-CSharp.dll"));
            Type scriptBase = vam.GetType("MVRScript", true);
            Assembly plugin = Assembly.LoadFile(pluginPath);
            Type[] types = plugin.GetTypes();
            int scripts = 0;
            for (int index = 0; index < types.Length; index++)
            {
                if (types[index] != scriptBase &&
                    scriptBase.IsAssignableFrom(types[index]))
                {
                    scripts++;
                    Console.WriteLine(
                        "MVRScript subtype: " + types[index].FullName);
                }
            }
            if (scripts != 1)
            {
                throw new InvalidOperationException(
                    "Expected exactly one MVRScript subtype; found " +
                    scripts + ".");
            }
            Console.WriteLine(
                "CLR " + Environment.Version +
                " Assembly.GetTypes validation passed for " +
                types.Length + " plugin type(s).");
            return 0;
        }
        catch (ReflectionTypeLoadException exception)
        {
            Console.Error.WriteLine(exception);
            Exception[] loaders = exception.LoaderExceptions;
            for (int index = 0; index < loaders.Length; index++)
            {
                Console.Error.WriteLine(
                    "LOADER " + index + ": " + loaders[index]);
            }
            return 1;
        }
        catch (Exception exception)
        {
            Console.Error.WriteLine(exception);
            return 1;
        }
    }

    private static Assembly ResolveAssembly(
        object sender,
        ResolveEventArgs arguments)
    {
        string name = new AssemblyName(arguments.Name).Name + ".dll";
        string path = Path.Combine(_managedDirectory, name);
        return File.Exists(path) ? Assembly.LoadFrom(path) : null;
    }
}

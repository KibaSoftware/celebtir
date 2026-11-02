// Offline SSH process double: never opens a socket or writes credentials.
using System;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;

public static class PasswordPeer
{
    public static int Main(string[] args)
    {
        Console.InputEncoding = new UTF8Encoding(false);
        Console.OutputEncoding = new UTF8Encoding(false);
        if (Environment.GetEnvironmentVariable("CELEBTIR_TEST_PEER_FAILURE") == "yes") return 42;
        string command = args[args.Length - 1];
        Match match = Regex.Match(command, "CELEBTIR_PASSWORD_READY_[a-zA-Z0-9_-]+");
        if (!match.Success) return 0; // preflight, upload preparation or cleanup
        if (!command.Contains("stty -echo") || !command.Contains("exec </dev/tty; exec \"$@\"")) return 12;
        string marker = match.Value;
        var passwordRead = Task.Factory.StartNew(() => Console.ReadLine());
        Console.Write(marker.Substring(0, 9)); Console.Out.Flush(); Thread.Sleep(30);
        if (passwordRead.IsCompleted) return 16; // Nothing may be sent before the complete readiness marker.
        Console.WriteLine(marker.Substring(9)); Console.Out.Flush();
        string password = passwordRead.GetAwaiter().GetResult();
        if (password == null) return 13;
        string hash;
        using (var sha = SHA256.Create()) hash = BitConverter.ToString(sha.ComputeHash(Encoding.UTF8.GetBytes(password))).Replace("-", "");
        if (hash != Environment.GetEnvironmentVariable("CELEBTIR_TEST_PASSWORD_HASH")) return 14;
        if (Environment.GetEnvironmentVariable("CELEBTIR_TEST_FOLLOWUP") == "yes" && Console.ReadLine() != "follow-up input") return 15;
        Console.WriteLine("PASS credential frame and subsequent input");
        Console.Error.WriteLine("PASS stderr forwarding");
        return 0;
    }
}

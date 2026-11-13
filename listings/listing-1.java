// SPDX-License-Identifier: CC0-1.0

package src.main.java.object_injection;

import java.io.*;

public class Listing1 {
  public static void main(String[] args) throws IOException, ClassNotFoundException {
    byte[] i1 = new byte[]{(byte)172, (byte)237, (byte)0, (byte)5, (byte)115, (byte)114, (byte)0, (byte)45, (byte)115, (byte)114, (byte)99, (byte)46, (byte)109, (byte)97, (byte)105, (byte)110, (byte)46, (byte)106, (byte)97, (byte)118, (byte)97, (byte)46, (byte)111, (byte)98, (byte)106, (byte)101, (byte)99, (byte)116, (byte)95, (byte)105, (byte)110, (byte)106, (byte)101, (byte)99, (byte)116, (byte)105, (byte)111, (byte)110, (byte)46, (byte)76, (byte)105, (byte)115, (byte)116, (byte)105, (byte)110, (byte)103, (byte)49, (byte)36, (byte)79, (byte)83, (byte)67, (byte)109, (byte)100, (byte)233, (byte)49, (byte)218, (byte)74, (byte)231, (byte)3, (byte)12, (byte)184, (byte)2, (byte)0, (byte)1, (byte)91, (byte)0, (byte)3, (byte)99, (byte)109, (byte)100, (byte)116, (byte)0, (byte)19, (byte)91, (byte)76, (byte)106, (byte)97, (byte)118, (byte)97, (byte)47, (byte)108, (byte)97, (byte)110, (byte)103, (byte)47, (byte)83, (byte)116, (byte)114, (byte)105, (byte)110, (byte)103, (byte)59, (byte)120, (byte)112, (byte)117, (byte)114, (byte)0, (byte)19, (byte)91, (byte)76, (byte)106, (byte)97, (byte)118, (byte)97, (byte)46, (byte)108, (byte)97, (byte)110, (byte)103, (byte)46, (byte)83, (byte)116, (byte)114, (byte)105, (byte)110, (byte)103, (byte)59, (byte)173, (byte)210, (byte)86, (byte)231, (byte)233, (byte)29, (byte)123, (byte)71, (byte)2, (byte)0, (byte)0, (byte)120, (byte)112, (byte)0, (byte)0, (byte)0, (byte)2, (byte)116, (byte)0, (byte)4, (byte)101, (byte)99, (byte)104, (byte)111, (byte)116, (byte)0, (byte)17, (byte)99, (byte)111, (byte)100, (byte)101, (byte)32, (byte)114, (byte)101, (byte)117, (byte)115, (byte)101, (byte)32, (byte)97, (byte)116, (byte)116, (byte)97, (byte)99, (byte)107};
    Cmd c = (Cmd) deserialize(i1);
    Runnable x = c::exec;
    x.run();
  }

  // ---

  static Object deserialize(byte[] data) throws IOException, ClassNotFoundException {
    ByteArrayInputStream bis = new ByteArrayInputStream(data);
    ObjectInputStream ois = new ObjectInputStream(bis);
    return ois.readObject();
  }

  static interface Cmd extends Serializable {
    public void exec();
  }

  static class Bkp implements Cmd {
    public void exec() {
      System.out.println("Backing up...");
    }
  }

  static class OSCmd implements Cmd {
    String[] cmd;
    public void exec() {
      try {
        new ProcessBuilder(this.cmd).inheritIO().start();
      } catch (IOException e) {
        System.out.println("OSCmd failed to execute");
      }
    }

    // ---

    OSCmd(String[] cmd) { this.cmd = cmd; }
  }

  // ---

  static void serialize_example() throws IOException {
    /*
     * usage:
     * 1. Comment out the contents of `main()`.
     * 2. Put `serialize_example();` in `main()`.
     * 3. Run the program.
     * 4. Remove `serialize_example();` from `main()`.
     * 5. Uncomment out the contents of `main()`.
     * 6. Replace the contents of `new byte[]{...}` with what was printed.
     */
    ByteArrayOutputStream bos = new ByteArrayOutputStream();
    ObjectOutputStream oos = new ObjectOutputStream(bos);
    oos.writeObject(new OSCmd(new String[]{"echo", "code reuse attack"}));
    oos.flush();
    byte[] data = bos.toByteArray();

    System.out.print("{");
    for (int i = 0; i < data.length; i++) {
      int unsigned = data[i] & 0xFF;
      System.out.print("(byte)" + unsigned);
      if (i < data.length - 1) System.out.print(", ");
    }
    System.out.println("};");
  }
}

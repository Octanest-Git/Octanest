using App;
using Xunit;

public class GreeterTests
{
    [Fact]
    public void Greets()
    {
        Assert.Equal("Hello, Octanest!", Greeter.Greet("Octanest"));
    }
}
